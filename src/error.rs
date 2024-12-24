use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::response::ApiResponse;

#[derive(Debug)]
pub enum ApiError {
    BadRequest,
    Unauthorized,
    NotFound,
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::NotFound => StatusCode::NOT_FOUND,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ApiError::BadRequest => "bad request".to_string(),
            ApiError::Unauthorized => "unauthorised".to_string(),
            ApiError::NotFound => "not found".to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let message = self.message();
        let response = ApiResponse::<()>::error(&message, status);
        (status, Json(response)).into_response()
    }
}
