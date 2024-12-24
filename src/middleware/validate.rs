use axum::{
    body::Body,
    http::{Request},
    middleware::Next,
    response::{IntoResponse},
};
use crate::error::ApiError;

pub async fn validate_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    let init_data = req
        .headers()
        .get("X-InitData")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::BadRequest)?;

    let decoded_init_data = urlencoding::decode(init_data)
        .map_err(|_| ApiError::BadRequest)?
        .into_owned();

    match crate::util::validator::validate_init_data(&decoded_init_data) {
        Ok(true) => {
            req.extensions_mut().insert(decoded_init_data);
            Ok(next.run(req).await)
        },
        Ok(false) => Err(ApiError::Unauthorized),
        Err(_) => Err(ApiError::BadRequest),
    }
}
