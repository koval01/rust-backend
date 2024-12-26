use bb8_redis::{
    bb8::{Pool, RunError},
    RedisConnectionManager,
    redis::{RedisError, AsyncCommands},
};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use std::future::Future;
use serde_json::{to_string, from_str};
use prisma_client_rust::QueryError;
use crate::error::ApiError;

#[derive(Debug)]
pub enum CacheError {
    Redis(RunError<RedisError>),
    Query(QueryError),
    NotFound,
}

impl From<RunError<RedisError>> for CacheError {
    fn from(err: RunError<RedisError>) -> Self {
        CacheError::Redis(err)
    }
}

impl From<QueryError> for CacheError {
    fn from(err: QueryError) -> Self {
        CacheError::Query(err)
    }
}

impl From<CacheError> for ApiError {
    fn from(err: CacheError) -> Self {
        match err {
            CacheError::Query(e) => ApiError::from(e),
            CacheError::Redis(e) => ApiError::Redis(e),
            CacheError::NotFound => ApiError::NotFound("Data not found".to_string()),
        }
    }
}

pub struct CacheWrapper<T> {
    redis_pool: Pool<RedisConnectionManager>,
    cache_ttl: Duration,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> CacheWrapper<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    pub fn new(redis_pool: Pool<RedisConnectionManager>, cache_ttl_secs: u64) -> Self {
        Self {
            redis_pool,
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn get_or_set<F>(
        &self,
        key: &str,
        db_fetch: F,
    ) -> Result<T, CacheError>
    where
        F: Future<Output = Result<Option<T>, QueryError>>,
    {
        let mut conn = self.redis_pool.get().await.map_err(CacheError::from)?;
        
        if let Ok(Some(cached_data)) = conn.get::<_, Option<String>>(key).await {
            if let Ok(parsed_data) = from_str(&cached_data) {
                return Ok(parsed_data);
            }
        }
        
        let db_result = db_fetch.await.map_err(CacheError::from)?;

        if let Some(data) = db_result {
            if let Ok(serialized) = to_string(&data) {
                let _: Result<(), _> = conn
                    .set_ex(key, serialized, self.cache_ttl.as_secs())
                    .await;
            }
            Ok(data)
        } else {
            Err(CacheError::NotFound)
        }
    }
}

#[macro_export]
macro_rules! cache_db_query {
    ($cache:expr, $key:expr, $query:expr) => {
        $cache.get_or_set($key, async { $query }).await.map_err(ApiError::from)
    };

    ($cache:expr, $key:expr, $query:expr, $error_handler:expr) => {
        $cache.get_or_set($key, async { $query }).await.map_err($error_handler)
    };
}
