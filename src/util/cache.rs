use bb8_redis::{
    bb8::{Pool, RunError},
    RedisConnectionManager,
    redis::{RedisError, AsyncCommands},
};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use std::future::Future;
use moka::future::Cache;
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

pub struct CacheWrapper<T> {
    redis_pool: Pool<RedisConnectionManager>,    // Redis connection pool
    moka_cache: Cache<String, String>,          // Moka in-memory cache
    cache_ttl: Duration,                        // Time-to-live for Redis cache
    _phantom: std::marker::PhantomData<T>,      // Marker for generic type T
}

// A generic wrapper for Redis-based caching
impl<T> CacheWrapper<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    /// Constructor for CacheWrapper
    pub fn new(
        redis_pool: Pool<RedisConnectionManager>,
        moka_cache: Cache<String, String>,
        cache_ttl_secs: u64,
    ) -> Self {
        Self {
            redis_pool,
            moka_cache,
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Attempts to retrieve the value from Moka, Redis, or database (via `db_fetch`).
    pub async fn get_or_set<F>(
        &self,
        key: &str,
        db_fetch: F,
    ) -> Result<T, CacheError>
    where
        F: Future<Output = Result<Option<T>, QueryError>>, // A future that fetches data from the database
    {
        // Step 1: Check Moka cache
        if let Some(cached_value) = self.moka_cache.get(key).await {
            if cached_value == "__not_found__" {
                return Err(CacheError::NotFound);
            }
            if let Ok(parsed_data) = from_str(&cached_value) {
                return Ok(parsed_data);
            }
        }

        // Step 2: Check Redis cache
        let mut conn = self.redis_pool.get().await.map_err(CacheError::from)?;
        if let Ok(Some(cached_data)) = conn.get::<_, Option<String>>(key).await {
            if cached_data == "__not_found__" {
                // Cache "not found" marker in Moka
                self.moka_cache.insert(key.to_string(), "__not_found__".to_string()).await;
                return Err(CacheError::NotFound);
            }
            if let Ok(parsed_data) = from_str(&cached_data) {
                // Cache the result in Moka
                self.moka_cache.insert(key.to_string(), cached_data).await;
                return Ok(parsed_data);
            }
        }

        // Step 3: Fetch from database
        let db_result = db_fetch.await.map_err(CacheError::from)?;

        if let Some(data) = db_result {
            // Cache the result in both Moka and Redis
            if let Ok(serialized) = to_string(&data) {
                self.moka_cache.insert(key.to_string(), serialized.clone()).await;
                let mut conn = self.redis_pool.get().await.map_err(CacheError::from)?;
                let _: Result<(), _> = conn.set_ex(key, serialized, self.cache_ttl.as_secs()).await;
            }
            Ok(data)
        } else {
            // Cache "not found" marker in both Moka and Redis
            self.cache_not_found(key).await?;
            Err(CacheError::NotFound)
        }
    }

    /// Caches a "not found" marker in both Moka and Redis
    pub async fn cache_not_found(&self, key: &str) -> Result<(), CacheError> {
        self.moka_cache.insert(key.to_string(), "__not_found__".to_string()).await;
        let mut conn = self.redis_pool.get().await.map_err(CacheError::from)?;
        let _: Result<(), _> = conn
            .set_ex(key, "__not_found__", self.cache_ttl.as_secs())
            .await;
        Ok(())
    }

    /// Updates the cache with new data for a given key in both Moka and Redis
    pub async fn set(&self, key: &str, data: &T) -> Result<(), CacheError> {
        let serialized = to_string(data).map_err(|_| CacheError::NotFound)?;

        // Check Moka cache first
        if let Some(cached_value) = self.moka_cache.get(key).await {
            // If cached data is the same as the new data, skip deletion
            if cached_value == serialized {
                return Ok(());
            }
        }

        // Update Moka cache
        self.moka_cache.insert(key.to_string(), serialized.clone()).await;

        // Update Redis cache without deleting
        let mut conn = self.redis_pool.get().await.map_err(CacheError::from)?;
        let _: Result<(), _> = conn
            .set_ex(key, serialized, self.cache_ttl.as_secs())
            .await;

        Ok(())
    }

    /// Deletes a key from both Moka and Redis
    #[allow(dead_code)]
    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.moka_cache.invalidate(key).await;
        let mut conn = self.redis_pool.get().await.map_err(CacheError::from)?;
        let _: Result<(), _> = conn.del(key).await;
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
