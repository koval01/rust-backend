use axum::{response::IntoResponse, Json};

use crate::{
    response::ApiResponse
};

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Hello from Duolang core!";
    let response: ApiResponse<()> = ApiResponse::message_only(MESSAGE);
    Json(response)
}
