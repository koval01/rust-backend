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

use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisError};

use reqwest::header::AUTHORIZATION;
use serde_json::json;
use std::env;

use crate::{
    error::ApiError,
    response::ApiResponse,
    util::cache::CacheError,
    model::GoogleUser,
    extractor::JwtKey
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
    Extension(redis_pool): Extension<Pool<RedisConnectionManager>>,
    Query(queries): Query<QueryAxumCallback>,
    Host(hostname): Host,
    Extension(jwt_key): Extension<JwtKey>,
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
            let client = reqwest::Client::new();
            let g_user = client.get("https://www.googleapis.com/oauth2/v1/userinfo")
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .send()
                .await.map_err(|e| ApiError::Custom(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .json::<GoogleUser>()
                .await.map_err(|e| ApiError::Custom(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            // delete after get userinfo
            let _: Result<(), _> = conn.del(queries.state).await;

            let user_map = g_user.to_btree_map();
            let token = jwt_key.sign(&user_map)
                .map_err(|e| ApiError::Custom(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let response = ApiResponse::success(json!({"jwt": token}));
            Ok((StatusCode::OK, Json(response)))
        }
        Err(_) => {
            Err(ApiError::BadRequest)
        }
    }
}
