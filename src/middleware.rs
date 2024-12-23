use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn validate_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let init_data = req
        .headers()
        .get("X-InitData")
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let decoded_init_data = urlencoding::decode(init_data)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .into_owned();

    match crate::validator::validate_init_data(&decoded_init_data) {
        Ok(true) => Ok(next.run(req).await),
        Ok(false) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}
