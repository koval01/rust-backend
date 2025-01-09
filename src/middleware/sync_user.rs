use axum::{body::Body, http::Request, middleware::Next, response::Response, Extension};

use crate::{
    cache_db_query, error::ApiError, extractor::InitData, model::User, prisma::*,
    util::cache::{CacheWrapper, CacheError},
};

use bb8_redis::{bb8::Pool, RedisConnectionManager};
use moka::future::Cache;

use std::sync::Arc;

pub async fn sync_user_middleware(
    InitData(init_user): InitData<User>,
    Extension(db): Extension<Arc<PrismaClient>>,
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
    Extension(moka_cache): Extension<Cache<String, String>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let cache = CacheWrapper::<user::Data>::new(redis_pool, moka_cache, 600);
    let redis_key = format!("user:{}", init_user.id);

    // Try to get user from cache first
    let cached_result = cache_db_query!(
        cache,
        &redis_key,
        db.user()
            .find_unique(user::id::equals(init_user.id))
            .exec()
            .await,
        @raw
    );

    let _ = match cached_result {
        Ok(existing_user) => {
            if needs_update(&init_user, &existing_user) {
                // Update user if needed
                let updated_user = db
                    .user()
                    .update(
                        user::id::equals(init_user.id),
                        vec![
                            user::first_name::set(init_user.first_name),
                            user::last_name::set(init_user.last_name),
                            user::username::set(init_user.username),
                            user::language_code::set(init_user.language_code),
                            user::SetParam::SetAllowsWriteToPm(init_user.allows_write_to_pm),
                            user::photo_url::set(init_user.photo_url),
                        ],
                    )
                    .exec()
                    .await?;

                // Update cache with new data
                let _ = cache.set(&redis_key, &updated_user).await;
                Ok(updated_user)
            } else {
                Ok(existing_user)
            }
        }
        Err(CacheError::NotFound) => {
            // Create new user if not found
            let new_user = db
                .user()
                .create(
                    init_user.id,
                    init_user.first_name,
                    init_user.language_code,
                    vec![
                        user::last_name::set(init_user.last_name),
                        user::username::set(init_user.username),
                        user::photo_url::set(init_user.photo_url),
                        user::SetParam::SetAllowsWriteToPm(init_user.allows_write_to_pm),
                    ],
                )
                .exec()
                .await?;

            // Cache the new user
            let _ = cache.set(&redis_key, &new_user).await;
            Ok(new_user)
        }
        Err(e) => Err(ApiError::from(e))
    };

    Ok(next.run(request).await)
}

#[inline(always)]
fn needs_update(init_user: &User, db_user: &user::Data) -> bool {
    init_user.first_name != db_user.first_name
        || init_user.last_name != db_user.last_name
        || init_user.username != db_user.username
        || init_user.language_code != db_user.language_code
        || init_user.allows_write_to_pm != db_user.allows_write_to_pm
        || init_user.photo_url != db_user.photo_url
}
