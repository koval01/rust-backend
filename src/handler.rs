use axum::{
    response::IntoResponse,
    body::Body,
    http::{Request, StatusCode},
    Json
};

use serde_json::{from_str};
use url::{form_urlencoded};

use crate::{
    error::ApiError,
    model::{User},
    response::UserResponse,
};

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "CRUD API in Rust using Axum";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn user_handler(req: Request<Body>) -> Result<impl IntoResponse, ApiError> {
    let decoded_init_data = req
        .extensions()
        .get::<String>()
        .ok_or(ApiError::BadRequest)?;

    let mut query_pairs = form_urlencoded::parse(decoded_init_data.as_bytes());

    let user_query = query_pairs.find(|(key, _)| key == "user");
    let user_query = user_query.ok_or(ApiError::BadRequest)?.1.to_string();

    let user: User = from_str(&user_query).map_err(|_| ApiError::BadRequest)?;

    let json_response = UserResponse {
        status: "success".to_string(),
        user,
    };
    Ok((StatusCode::OK, Json(json_response)))
}
