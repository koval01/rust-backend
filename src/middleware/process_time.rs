use axum::{
    middleware::Next,
    response::Response,
    http::{Request, HeaderValue},
    body::Body,
};
use std::time::Instant;

pub async fn process_time_middleware(
    request: Request<Body>,
    next: Next,
) -> Response {
    let start_time = Instant::now();

    let response = next.run(request).await;

    let process_time = start_time.elapsed().as_millis();
    let process_time_header = format!("{} ms", process_time);

    let mut response = response;
    response.headers_mut().insert(
        "x-process-time",
        HeaderValue::from_str(&process_time_header).unwrap(),
    );

    response
}
