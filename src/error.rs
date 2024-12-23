use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    BadRequest,
    Unauthorized,
    NotFound,
    Conflict,
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::Conflict => StatusCode::CONFLICT,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ApiError::BadRequest => "bad request".to_string(),
            ApiError::Unauthorized => "unauthorised".to_string(),
            ApiError::NotFound => "not found".to_string(),
            ApiError::Conflict => "conflict".to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let code = status.as_u16();
        let message = self.message();
        (status, Json(json!({ "status": "error", "code": code, "message": message }))).into_response()
    }
}
