use axum::{
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use crate::{
    error::ApiError,
    model::User,
    response::{ApiResponse, UserResponseData},
    extractor::InitData,
};

pub async fn user_handler(
    InitData(user): InitData<User>,
) -> Result<impl IntoResponse, ApiError> {
    let user_data = UserResponseData { user };
    let response = ApiResponse::success(user_data);
    Ok((StatusCode::OK, Json(response)))
}
