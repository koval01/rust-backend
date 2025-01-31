use axum::{
    Extension, Json,
    extract::Query,
    http::StatusCode,
    response::IntoResponse
};
use oauth_axum::{
    CustomProvider, OAuthClient,
    providers::google::GoogleProvider
};
use axum_extra::extract::Host;

use moka::future::Cache;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisError};

use reqwest::header::AUTHORIZATION;
use serde_json::json;

use std::{env, sync::Arc};
use tracing::{debug, error, info};

use crate::{
    cache_db_query,
    error::ApiError,
    model::GoogleUser, 
    extractor::JwtKey,
    response::ApiResponse,
    prisma::{PrismaClient, user},
    util::cache::{CacheError, CacheWrapper}
};

#[derive(Clone, serde::Deserialize)]
pub struct QueryAxumCallback {
    pub code: String,
    pub state: String,
}

fn get_client(hostname: String) -> CustomProvider {
    let protocol = if hostname.starts_with("localhost") || hostname.starts_with("127.0.0.1") {
        "http"
    } else {
        "https"
    };

    GoogleProvider::new(
        env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
        env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET must be set"),
        format!("{}://{}/api/v1/auth/callback", protocol, hostname),
    )
}

pub async fn login(
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
    Host(hostname): Host
) -> Result<impl IntoResponse, ApiError> {
    let client = get_client(hostname);

    let mut conn = redis_pool.get().await.map_err(CacheError::from)?;
    let state_oauth = client
        .generate_url(
            Vec::from([String::from("openid"), String::from("profile"), String::from("email")]),
            |state_e| async move {
                let _: Result<(), _> = conn.set_ex(state_e.state, state_e.verifier, 300).await;
            },
        )
        .await
        .ok()
        .unwrap()
        .state
        .unwrap();

    let authorize_url = state_oauth.url_generated.unwrap();

    let response = ApiResponse::success(json!({"url": authorize_url}));
    Ok((StatusCode::OK, Json(response)))
}

pub async fn callback(
    Query(queries): Query<QueryAxumCallback>,
    Host(hostname): Host,
    Extension(jwt_key): Extension<JwtKey>,
    Extension(db): Extension<Arc<PrismaClient>>,
    Extension(moka_cache): Extension<Cache<String, String>>,
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
) -> Result<impl IntoResponse, ApiError> {
    let mut conn = redis_pool.get().await.map_err(CacheError::from)?;
    let item = conn.get::<_, Option<String>>(queries.state.clone()).await.map_err(RedisError::from)?;
    let client = get_client(hostname);
    let verifier = item.unwrap_or_default();
    let token_result = client
        .generate_token(queries.code, verifier)
        .await;

    match token_result {
        Ok(token) => {
            let _: Result<(), _> = conn.del(queries.state).await;

            let client = reqwest::Client::new();
            let g_user = client.get("https://www.googleapis.com/oauth2/v1/userinfo")
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .send()
                .await.map_err(|e| { 
                    error!("Failed to query the google server to retrieve user data: {:?}", e);
                    ApiError::Custom(StatusCode::INTERNAL_SERVER_ERROR, String::from("Failed to query the google server to retrieve user data")) 
                })?
                .json::<GoogleUser>()
                .await.map_err(|e| {
                    error!("Failed to decode google userinfo: {}", e);
                    ApiError::Custom(StatusCode::INTERNAL_SERVER_ERROR, String::from("Failed to decode google userinfo")) 
                })?;

            let cache = CacheWrapper::<user::Data>::new(redis_pool.clone(), moka_cache, 30);
            let redis_key = format!("user:{}", g_user.sub.to_string());

            let cached_result = cache_db_query!(
                cache,
                &redis_key,
                db.user()
                    .find_first(vec![user::google_id::equals(g_user.sub.clone())])
                    .exec()
                    .await,
                @raw
            );

            let _ = match cached_result {
                Ok(existing_user) => Ok(existing_user),
                Err(CacheError::NotFound) => {
                    info!("Creating user {} in database", &g_user.sub);
                    
                    // Create new user if not found
                    let new_user = db
                        .user()
                        .create(
                            g_user.sub.to_string(),
                            g_user.given_name.to_string(),
                            vec![
                                user::photo_url::set(g_user.picture.clone()),
                            ]
                        )
                        .exec()
                        .await
                        .map_err(|e| { 
                            error!("Failed to create user {} in database. {:?}", &g_user.sub, &e);
                            ApiError::Database(e) 
                        })?;

                    // Cache the new user
                    let _ = cache.set(&redis_key, &new_user).await;
                    Ok(new_user)
                }
                Err(e) => { 
                    error!("Error fetching user {} from database. {:?}", &g_user.sub, &e);
                    Err(ApiError::from(e)) 
                }
            };
            
            let user_map = g_user.to_btree_map();
            let token = jwt_key.sign(&user_map)
                .map_err(|e| { 
                    error!("Failed to create a jwt token for user {}. {:?}", &g_user.sub, &e);
                    ApiError::Custom(StatusCode::INTERNAL_SERVER_ERROR, String::from("Failed to create a jwt token")) 
                })?;
            let response = ApiResponse::success(json!({"jwt": token}));
            Ok((StatusCode::OK, Json(response)))
        }
        Err(_) => {
            debug!("An error occurred while processing the google authorization token.");
            Err(ApiError::BadRequest)
        }
    }
}
