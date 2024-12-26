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
    Redis(RunError<RedisError>), // Error related to Redis connection or operations
    Query(QueryError),           // Error related to database queries
    NotFound,                    // Error indicating that the data was not found
}

// Implement conversion from Redis errors to CacheError
impl From<RunError<RedisError>> for CacheError {
    fn from(err: RunError<RedisError>) -> Self {
        CacheError::Redis(err)
    }
}

// Implement conversion from QueryError (database error) to CacheError
impl From<QueryError> for CacheError {
    fn from(err: QueryError) -> Self {
        CacheError::Query(err)
    }
}

// Convert CacheError into ApiError for unified error handling in the API layer
impl From<CacheError> for ApiError {
    fn from(err: CacheError) -> Self {
        match err {
            CacheError::Query(e) => ApiError::from(e), // Convert database errors
            CacheError::Redis(e) => ApiError::Redis(e), // Convert Redis errors
            CacheError::NotFound => ApiError::NotFound("Data not found".to_string()), // Convert not found errors
        }
    }
}

// A generic wrapper for Redis-based caching
pub struct CacheWrapper<T> {
    redis_pool: Pool<RedisConnectionManager>, // Redis connection pool
    cache_ttl: Duration,                     // Time-to-live for cached data
    _phantom: std::marker::PhantomData<T>,   // Marker for generic type T
}

impl<T> CacheWrapper<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    // Constructor for CacheWrapper
    pub fn new(redis_pool: Pool<RedisConnectionManager>, cache_ttl_secs: u64) -> Self {
        Self {
            redis_pool,
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Attempts to retrieve the value from cache. If not found, fetches it from the database
    /// and caches the result. If the value does not exist in the database, caches a "not found" marker.
    pub async fn get_or_set<F>(
        &self,
        key: &str,
        db_fetch: F,
    ) -> Result<T, CacheError>
    where
        F: Future<Output = Result<Option<T>, QueryError>>, // A future that fetches data from the database
    {
        let mut conn = self.redis_pool.get().await.map_err(CacheError::from)?;

        // Check if the key exists in Redis
        if let Ok(Some(cached_data)) = conn.get::<_, Option<String>>(key).await {
            if cached_data == "__not_found__" {
                // If the key is marked as "not found", return a NotFound error
                return Err(CacheError::NotFound);
            }
            if let Ok(parsed_data) = from_str(&cached_data) {
                // If the cached data is valid, return it
                return Ok(parsed_data);
            }
        }

        // If the data is not in the cache, fetch it from the database
        let db_result = db_fetch.await.map_err(CacheError::from)?;

        if let Some(data) = db_result {
            // If the data is found, cache it in Redis
            if let Ok(serialized) = to_string(&data) {
                let _: Result<(), _> = conn
                    .set_ex(key, serialized, self.cache_ttl.as_secs())
                    .await;
            }
            Ok(data)
        } else {
            // If the data is not found, cache the "not found" marker
            self.cache_not_found(key).await?;
            Err(CacheError::NotFound)
        }
    }

    /// Caches a "not found" marker for a given key to prevent repeated database queries
    pub async fn cache_not_found(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.redis_pool.get().await.map_err(CacheError::from)?;
        let _: Result<(), _> = conn
            .set_ex(key, "__not_found__", self.cache_ttl.as_secs()) // Cache the "not found" marker
            .await;
        Ok(())
    }

    /// Updates the cache with new data for a given key
    pub async fn set(&self, key: &str, data: &T) -> Result<(), CacheError> {
        let mut conn = self.redis_pool.get().await.map_err(CacheError::from)?;
        let serialized = to_string(data).map_err(|_| CacheError::NotFound)?; // Serialize the data
        let _: Result<(), _> = conn
            .set_ex(key, serialized, self.cache_ttl.as_secs()) // Cache the serialized data
            .await;
        Ok(())
    }

    /// Deletes a key from the cache
    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.redis_pool.get().await.map_err(CacheError::from)?;
        let _: Result<(), _> = conn.del(key).await; // Remove the key from Redis
        Ok(())
    }
}

// Macro to simplify cache usage in database queries
#[macro_export]
macro_rules! cache_db_query {
    ($cache:expr, $key:expr, $query:expr) => {
        $cache.get_or_set($key, async { $query }).await.map_err(ApiError::from)
    };

    ($cache:expr, $key:expr, $query:expr, $error_handler:expr) => {
        $cache.get_or_set($key, async { $query }).await.map_err($error_handler)
    };
}
