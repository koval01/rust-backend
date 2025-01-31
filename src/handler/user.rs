use axum::{
    extract::Path,
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::{
    error::ApiError,
    model::GoogleUser,
    prisma::*,
    response::ApiResponse,
    util::cache::CacheWrapper,
    Extension,
    cache_db_query
};
use bb8_redis::{bb8::Pool, RedisConnectionManager};
use moka::future::Cache;

type Database = Extension<Arc<PrismaClient>>;

macro_rules! get_user {
    ($cache:expr, $db:expr, $id:expr, @google_id) => {
        get_user_internal!($cache, $db, $id, user::google_id::equals($id))
    };
    ($cache:expr, $db:expr, $id:expr) => {
        get_user_internal!($cache, $db, $id, user::id::equals($id))
    };
}

macro_rules! get_user_internal {
    ($cache:expr, $db:expr, $id:expr, $condition:expr) => {
        cache_db_query!(
            $cache,
            &format!("user:{}", $id),
            $db.user()
                .find_first(vec![$condition])
                .exec()
                .await,
            |_| ApiError::NotFound("User does not exist".to_string())
        )
    };
}

/// Handles GET requests for the authenticated user's profile
pub async fn user_handler_get(
    user: GoogleUser,
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
    Extension(moka_cache): Extension<Cache<String, String>>,
    db: Database
) -> Result<impl IntoResponse, ApiError> {
    let user_id = user.sub;
    let cache = CacheWrapper::<user::Data>::new(redis_pool, moka_cache, 600);

    // Attempt to fetch the user from cache or database
    let user = get_user!(cache, db, user_id, @google_id)?;

    let response = ApiResponse::success(user);
    Ok((StatusCode::OK, Json(response)))
}

/// Handles GET requests for a user by ID
pub async fn user_id_handler_get(
    _: GoogleUser,
    Path(id): Path<uuid::Uuid>,
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
    Extension(moka_cache): Extension<Cache<String, String>>,
    db: Database
) -> Result<impl IntoResponse, ApiError> {
    let cache = CacheWrapper::<user::Data>::new(redis_pool, moka_cache, 600);

    // Attempt to fetch the user from cache or database
    let user = get_user!(cache, db, id.to_string())?;

    let response = ApiResponse::success(user);
    Ok((StatusCode::OK, Json(response)))
}
