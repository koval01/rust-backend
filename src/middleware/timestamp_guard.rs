use axum::{
    middleware::Next,
    response::IntoResponse,
    http::{Request, Method},
    body::Body,
};
use chrono::Utc;
use crate::error::ApiError;

pub async fn timestamp_guard_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    if request.method() == Method::OPTIONS {
        return Ok(next.run(request).await);
    }

    let current_timestamp = Utc::now().timestamp() as u64;

    let timestamp_header = request
        .headers()
        .get("x-timestamp")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::BadRequest)?;

    let request_timestamp = timestamp_header
        .parse::<u64>()
        .map_err(|_| ApiError::BadRequest)?;

    const MAX_TIME_DIFF: u64 = 30;

    if current_timestamp.abs_diff(request_timestamp) > MAX_TIME_DIFF {
        return Err(ApiError::Forbidden);
    }

    Ok(next.run(request).await)
}
