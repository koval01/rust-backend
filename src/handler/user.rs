use axum::{
    extract::{Path, rejection::PathRejection},
    response::IntoResponse,
    http::StatusCode,
    Json,
    Extension,
};

use bb8_redis::{bb8::Pool, RedisConnectionManager};
use moka::future::Cache;
use reqwest::Client;

use crate::{
    error::ApiError,
    response::ApiResponse,
    util::cache::{CacheWrapper, JsonResponseExt},
    cache_http_request,
};
use crate::model::User;

/// Handles GET requests for all users from JSONPlaceholder
pub async fn users_handler_get(
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
    Extension(moka_cache): Extension<Cache<String, String>>,
    Extension(http_client): Extension<Client>,
) -> Result<impl IntoResponse, ApiError> {
    // Create a cache wrapper for User vector
    let cache = CacheWrapper::<Vec<User>>::new(
        redis_pool,
        moka_cache,
        10,
        http_client,
    );

    // Attempt to fetch users from cache or JSONPlaceholder API
    let users = cache_http_request!(
        cache, 
        "users:all",
        |client: Client| async move {
            client.get("https://jsonplaceholder.typicode.com/users")
                .send()
                .await?
                .json_cached::<Vec<User>>()
                .await
        }
    )?;

    let response = ApiResponse::success(users);
    Ok((StatusCode::OK, Json(response)))
}

/// Handles GET requests for a specific user by ID from JSONPlaceholder
pub async fn user_id_handler_get(
    id: Result<Path<i32>, PathRejection>,
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
    Extension(moka_cache): Extension<Cache<String, String>>,
    Extension(http_client): Extension<Client>,
) -> Result<impl IntoResponse, ApiError> {
    let Path(id) = id.map_err(|e| ApiError::Conflict(e.to_string()))?;

    // Create a cache wrapper for a single User
    let cache = CacheWrapper::<User>::new(
        redis_pool,
        moka_cache,
        10,
        http_client,
    );

    // Attempt to fetch the user from cache or JSONPlaceholder API
    let user = cache_http_request!(
        cache, 
        &format!("user:{}", id),
        |client: Client| async move {
            client.get(format!("https://jsonplaceholder.typicode.com/users/{}", id))
                .send()
                .await?
                .json_cached::<User>()
                .await
        }
    )?;

    let response = ApiResponse::success(user);
    Ok((StatusCode::OK, Json(response)))
}
