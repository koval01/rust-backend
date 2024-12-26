use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
    Extension,
};

use bb8::{Pool, RunError};
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use serde_json::{from_str, to_string};
use std::sync::Arc;
use crate::{
    prisma::*,
    model::User,
    error::ApiError,
    extractor::InitData,
};

pub async fn sync_user_middleware(
    InitData(init_user): InitData<User>,
    Extension(db): Extension<Arc<PrismaClient>>,
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let redis_key = format!("user:{}", init_user.id);

    let mut conn = redis_pool.get().await.map_err(|e| ApiError::Redis(e))?;
    let cached_user = conn.get::<_, Option<String>>(&redis_key).await.map_err(|e| ApiError::Redis(RunError::from(e)))?;

    if let Some(cached_user) = cached_user {
        if let Ok(_) = from_str::<user::Data>(&cached_user) {
            return Ok(next.run(request).await);
        }
    }

    let db_user = db
        .user()
        .find_unique(user::id::equals(init_user.id))
        .exec()
        .await?;

    let user_data = match db_user {
        Some(existing_user) => {
            if needs_update(&init_user, &existing_user) {
                let updated_user = db.user()
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
                updated_user
            } else {
                existing_user
            }
        }
        None => {
            let new_user = db.user()
                .create(
                    init_user.id,
                    init_user.first_name,
                    init_user.language_code,
                    init_user.allows_write_to_pm,
                    vec![
                        user::last_name::set(init_user.last_name),
                        user::username::set(init_user.username),
                        user::photo_url::set(init_user.photo_url),
                    ],
                )
                .exec()
                .await?;
            new_user
        }
    };

    if let Ok(serialized_user) = to_string(&user_data) {
        let mut conn = redis_pool.get().await.map_err(|e| ApiError::Redis(e))?;
        let _: Result<(), _> = conn.set_ex(&redis_key, serialized_user, 300).await;
    }

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
