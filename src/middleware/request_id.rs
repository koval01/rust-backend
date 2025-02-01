use axum::{
    middleware::Next,
    response::Response,
    http::{Request, HeaderValue},
    body::Body,
};
use sentry::Scope;
use tracing::{debug_span, Instrument};
use uuid::Uuid;

pub async fn request_id_middleware(
    request: Request<Body>,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();
    
    sentry::configure_scope(|scope: &mut Scope | {
        scope.set_tag("request_id", &request_id);
    });
    
    let span = debug_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri()
    );

    let response = next.run(request).instrument(span).await;

    let mut response = response;
    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&request_id).unwrap(),
    );

    response
}
