use axum::{
    response::IntoResponse,
    http::StatusCode,
    Json
};

use crate::{
    error::ApiError,
    model::{User},
    response::UserResponse,
    extractor::InitData,
};

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "CRUD API in Rust using Axum";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn user_handler(
    InitData(user): InitData<User>,
) -> Result<impl IntoResponse, ApiError> {
    let json_response = UserResponse {
        status: "success".to_string(),
        user,
    };
    Ok((StatusCode::OK, Json(json_response)))
}
