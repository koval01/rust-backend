use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    extract::rejection::QueryRejection,
    Json,
};
use axum::extract::rejection::PathRejection;

use bb8::RunError;
use redis::RedisError;

use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;

use tracing::debug;
use crate::response::ApiResponse;
use crate::util::cache::CacheError;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ApiError {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound(String),
    Conflict(String),
    Timeout,
    InternalServerError,
    Redis(RunError<RedisError>),
    Reqwest(ReqwestError),
    Serialization(SerdeJsonError),
    Custom(StatusCode, String),
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::Timeout => StatusCode::GATEWAY_TIMEOUT,
            ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Reqwest(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Serialization(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Custom(code, _) => *code,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ApiError::BadRequest => "bad request".to_string(),
            ApiError::Unauthorized => "unauthorised".to_string(),
            ApiError::Forbidden => "forbidden".to_string(),
            ApiError::NotFound(error) => if error.is_empty() { "not found".to_string() } else { error.clone() },
            ApiError::Conflict(error) => if error.is_empty() { "conflict".to_string() } else { error.clone() },
            ApiError::Timeout => "request timed out".to_string(),
            ApiError::InternalServerError => "internal error".to_string(),
            ApiError::Redis(error) => format!("redis error: {}", error),
            ApiError::Reqwest(error) => format!("HTTP request error: {}", error),
            ApiError::Serialization(error) => format!("JSON serialization error: {}", error),
            ApiError::Custom(_, message) => message.clone(),
        }
    }
}

impl From<RedisError> for ApiError {
    fn from(error: RedisError) -> Self {
        debug!("{:#?}", error);
        ApiError::Redis(RunError::User(error))
    }
}

impl From<ReqwestError> for ApiError {
    fn from(error: ReqwestError) -> Self {
        debug!("Reqwest error: {:#?}", error);
        ApiError::Reqwest(error)
    }
}

impl From<SerdeJsonError> for ApiError {
    fn from(error: SerdeJsonError) -> Self {
        debug!("Serialization error: {:#?}", error);
        ApiError::Serialization(error)
    }
}

impl From<QueryRejection> for ApiError {
    fn from(error: QueryRejection) -> Self {
        debug!("{:#?}", error);
        ApiError::Custom(StatusCode::BAD_REQUEST, error.body_text()) 
    }
}

impl From<PathRejection> for ApiError {
    fn from(error: PathRejection) -> Self {
        debug!("{:#?}", error);
        ApiError::Custom(StatusCode::BAD_REQUEST, error.body_text())
    }
}

impl From<CacheError> for ApiError {
    fn from(err: CacheError) -> Self {
        match err {
            CacheError::Redis(e) => ApiError::Redis(e),
            CacheError::Reqwest(e) => ApiError::Reqwest(e),
            CacheError::Serialization(e) => ApiError::Serialization(e),
            CacheError::NotFound => ApiError::NotFound("Resource not found".to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.message();
        let response = ApiResponse::<()>::error(&message, status);
        (status, Json(response)).into_response()
    }
}
