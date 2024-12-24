use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use prisma_client_rust::QueryError;
use crate::response::ApiResponse;

#[derive(Debug)]
pub enum ApiError {
    BadRequest,
    Unauthorized,
    NotFound,
    Database(QueryError),
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ApiError::BadRequest => "bad request".to_string(),
            ApiError::Unauthorized => "unauthorised".to_string(),
            ApiError::NotFound => "not found".to_string(),
            ApiError::Database(error) => format!("database error: {}", error),
        }
    }
}

impl From<QueryError> for ApiError {
    fn from(error: QueryError) -> Self {
        ApiError::Database(error)
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
