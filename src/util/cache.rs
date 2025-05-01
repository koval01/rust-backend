use bb8_redis::{
    bb8::{Pool, RunError},
    RedisConnectionManager,
    redis::{RedisError, AsyncCommands},
};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{to_string, from_str};

use moka::future::Cache;
use reqwest::{Client, Error as ReqwestError};

use std::time::Duration;
use std::future::Future;

#[derive(Debug)]
pub enum CacheError {
    Redis(RunError<RedisError>),      // Error related to Redis connection or operations
    Reqwest(ReqwestError),            // Error related to HTTP requests
    Serialization(serde_json::Error), // Error related to JSON serialization/deserialization
    NotFound,                         // Error indicating that the data was not found
}

// Implement conversion from Redis errors to CacheError
impl From<RunError<RedisError>> for CacheError {
    fn from(err: RunError<RedisError>) -> Self {
        CacheError::Redis(err)
    }
}

// Implement conversion from Reqwest errors to CacheError
impl From<ReqwestError> for CacheError {
    fn from(err: ReqwestError) -> Self {
        CacheError::Reqwest(err)
    }
}

// Implement conversion from JSON errors to CacheError
impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        CacheError::Serialization(err)
    }
}

pub struct CacheWrapper<T> {
    redis_pool: Pool<RedisConnectionManager>,   // Redis connection pool
    moka_cache: Cache<String, String>,          // Moka in-memory cache
    cache_ttl: Duration,                        // Time-to-live for Redis cache
    http_client: Client,                        // Reqwest HTTP client
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
        http_client: Client,
    ) -> Self {
        Self {
            redis_pool,
            moka_cache,
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            http_client,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the HTTP client
    #[allow(dead_code)]
    pub fn client(&self) -> &Client {
        &self.http_client
    }

    /// Attempts to retrieve the value from Moka, Redis, or HTTP (via `http_fetch`).
    pub async fn get_or_fetch<F, Fut>(
        &self,
        key: &str,
        http_fetch: F,
    ) -> Result<T, CacheError>
    where
        F: FnOnce(Client) -> Fut,
        Fut: Future<Output = Result<Option<T>, ReqwestError>> + Send,
    {
        // Check Moka cache
        if let Some(cached_value) = self.moka_cache.get(key).await {
            if cached_value == "__not_found__" {
                return Err(CacheError::NotFound);
            }
            if let Ok(parsed_data) = from_str(&cached_value) {
                return Ok(parsed_data);
            }
        }

        // Check Redis cache
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

        // Fetch from HTTP request
        // Use clone of the client to avoid lifetime issues
        let client_clone = self.http_client.clone();
        let http_result = http_fetch(client_clone).await.map_err(CacheError::from)?;

        if let Some(data) = http_result {
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
    #[allow(dead_code)]
    pub async fn set(&self, key: &str, data: &T) -> Result<(), CacheError> {
        let serialized = to_string(data).map_err(|e| CacheError::Serialization(e))?;

        // Check Moka cache first
        if let Some(cached_value) = self.moka_cache.get(key).await {
            // If cached data is the same as the new data, skip deletion
            if cached_value == serialized {
                return Ok(());
            }
        }

        // Update Moka cache
        self.moka_cache.insert(key.to_string(), serialized.clone()).await;

        // Update Redis cache
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

#[macro_export]
macro_rules! cache_http_request {
    ($cache:expr, $key:expr, $request:expr) => {
        $cache.get_or_fetch($key, |client| {
            let fut = async move {
                $request(client).await
            };
            fut
        }).await
    };

    ($cache:expr, $key:expr, $request:expr, $error_handler:expr) => {
        $cache.get_or_fetch($key, |client| {
            let fut = async move {
                $request(client).await
            };
            fut
        }).await.map_err($error_handler)
    };
}

pub trait JsonResponseExt {
    async fn json_cached<T>(self) -> Result<Option<T>, ReqwestError>
    where
        T: DeserializeOwned;
}

impl JsonResponseExt for reqwest::Response {
    async fn json_cached<T>(self) -> Result<Option<T>, ReqwestError>
    where
        T: DeserializeOwned,
    {
        if self.status().is_success() {
            self.json::<T>().await.map(Some)
        } else if self.status().is_client_error() {
            // 4xx responses - treat as "not found"
            Ok(None)
        } else {
            // Other errors - propagate
            self.error_for_status()?.json::<T>().await.map(Some)
        }
    }
}
