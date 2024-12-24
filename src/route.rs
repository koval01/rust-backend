use axum::{
    middleware,
    routing::{get},
    Router,
    response::IntoResponse,
};
use tower::ServiceBuilder;

use crate::{
    handler::{health_checker_handler, user_handler},
    middleware::validate_middleware,
    error::ApiError,
};

pub fn create_router() -> Router {

    // Routes without middleware
    let public_routes = Router::new()
        .route("/api/health", get(health_checker_handler));

    // Routes with middleware
    let protected_routes = Router::new()
        .route(
            "/api/user",
            get(user_handler),
        )
        .layer(ServiceBuilder::new().layer(middleware::from_fn(validate_middleware)));

    // Merge routes and add shared state and fallback
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback(|| async { ApiError::NotFound.into_response() })
}
