use axum::{
    extract::Path,
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::{
    error::ApiError,
    model::User,
    response::{ApiResponse, UserResponseData},
    util::cache::CacheWrapper,
    extractor::InitData,
    Extension,
    cache_db_query
};
use bb8_redis::{bb8::Pool, RedisConnectionManager};
use moka::future::Cache;
use crate::prisma::*;

type Database = Extension<Arc<PrismaClient>>;

/// Handles GET requests for the authenticated user's profile
pub async fn user_handler_get(
    InitData(user): InitData<User>,
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
    Extension(moka_cache): Extension<Cache<String, String>>,
    db: Database
) -> Result<impl IntoResponse, ApiError> {
    let user_data = UserResponseData { user };
    let user_id = user_data.user.id;
    let cache = CacheWrapper::<user::Data>::new(redis_pool, moka_cache, 600);

    // Attempt to fetch the user from cache or database
    let user = cache_db_query!(
        cache,
        &format!("user:{}", user_id),
        db.user()
            .find_first(vec![user::id::equals(user_id)])
            .exec()
            .await
    )?;

    let response = ApiResponse::success(user);
    Ok((StatusCode::OK, Json(response)))
}

/// Handles GET requests for a user by ID
pub async fn user_id_handler_get(
    Path(id): Path<i64>,
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
    Extension(moka_cache): Extension<Cache<String, String>>,
    db: Database
) -> Result<impl IntoResponse, ApiError> {
    let cache = CacheWrapper::<user::Data>::new(redis_pool, moka_cache, 600);

    // Attempt to fetch the user from cache or database
    let user = cache_db_query!(
        cache,
        &format!("user:{}", id),
        db.user()
            .find_first(vec![user::id::equals(id)])
            .exec()
            .await
    )?;

    let response = ApiResponse::success(user);
    Ok((StatusCode::OK, Json(response)))
}

/// Handles POST requests to create a new user
pub async fn user_handler_post(
    InitData(user): InitData<User>,
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
    Extension(moka_cache): Extension<Cache<String, String>>,
    db: Database
) -> Result<impl IntoResponse, ApiError> {
    let User {
        id,
        first_name,
        last_name,
        username,
        language_code,
        allows_write_to_pm,
        photo_url,
    } = user;

    let cache = CacheWrapper::<user::Data>::new(redis_pool.clone(), moka_cache, 600);

    // Create the user in the database
    let data = db.user()
        .create(
            id,
            first_name,
            language_code,
            allows_write_to_pm,
            vec![
                user::last_name::set(last_name),
                user::username::set(username),
                user::photo_url::set(photo_url),
            ],
        )
        .exec()
        .await?;

    // Remove "not found" cache if it exists and cache the new user
    let _ = cache.set(&format!("user:{}", id), &data).await;

    let response = ApiResponse::success(data);
    Ok((StatusCode::OK, Json(response)))
}

/// Handles PUT requests to update an existing user
pub async fn user_handler_put(
    InitData(user): InitData<User>,
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
    Extension(moka_cache): Extension<Cache<String, String>>,
    db: Database
) -> Result<impl IntoResponse, ApiError> {
    let User {
        id,
        first_name,
        last_name,
        username,
        language_code,
        allows_write_to_pm,
        photo_url,
    } = user;

    let cache = CacheWrapper::<user::Data>::new(redis_pool.clone(), moka_cache, 600);

    // Update the user in the database
    let data = db
        .user()
        .update(
            user::id::equals(id),
            vec![
                user::first_name::set(first_name),
                user::last_name::set(last_name),
                user::username::set(username),
                user::language_code::set(language_code),
                user::SetParam::SetAllowsWriteToPm(allows_write_to_pm),
                user::photo_url::set(photo_url),
            ]
        )
        .exec()
        .await?;

    // Update the cache with the new user data
    let _ = cache.set(&format!("user:{}", id), &data).await;

    let response = ApiResponse::success(data);
    Ok((StatusCode::OK, Json(response)))
}
