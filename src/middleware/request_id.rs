use axum::{
    middleware::Next,
    response::Response,
    http::{Request, HeaderValue},
    body::Body,
};
use uuid::Uuid;

pub async fn request_id_middleware(
    request: Request<Body>,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();

    sentry::configure_scope(|scope| {
        scope.set_tag("request_id", &request_id);
    });

    let mut response = next.run(request).await;

    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&request_id).unwrap(),
    );

    response
}
