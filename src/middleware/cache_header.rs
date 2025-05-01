use axum::{
    middleware::Next,
    response::Response,
    http::{Request, HeaderValue},
    body::Body,
};

pub async fn cache_header_middleware(
    request: Request<Body>,
    next: Next,
) -> Response {
    let response = next.run(request).await;

    let mut response = response;
    response.headers_mut().insert(
        "cache-control",
        HeaderValue::from_str("public, max-age=10, stale-while-revalidate=10").unwrap(),
    );

    response
}
