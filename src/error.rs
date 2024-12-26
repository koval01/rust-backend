use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bb8::RunError;
use prisma_client_rust::QueryError;
use redis::RedisError;
use crate::response::ApiResponse;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ApiError {
    BadRequest,
    Unauthorized,
    NotFound(String),
    Timeout,
    Database(QueryError),
    Redis(RunError<RedisError>),
    InternalServerError,
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Timeout => StatusCode::GATEWAY_TIMEOUT,
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ApiError::BadRequest => "bad request".to_string(),
            ApiError::Unauthorized => "unauthorised".to_string(),
            ApiError::NotFound(error) => if error.is_empty() { "not found".to_string() } else { error.clone() },
            ApiError::Timeout => "request timed out".to_string(),
            ApiError::Database(error) => format!("database error: {}", error),
            ApiError::Redis(error) => format!("redis error: {}", error),
            ApiError::InternalServerError => "internal error".to_string(),
        }
    }
}

impl From<QueryError> for ApiError {
    fn from(error: QueryError) -> Self {
        ApiError::Database(error)
    }
}

impl From<RedisError> for ApiError {
    fn from(error: RedisError) -> Self {
        ApiError::Redis(RunError::User(error))
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(_: serde_json::Error) -> Self {
        ApiError::InternalServerError
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
