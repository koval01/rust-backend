use axum::{
    routing::{get},
    Router,
    response::IntoResponse,
};

use tower::ServiceBuilder;

use crate::{
    handler::{
        health_checker_handler,
        users_handler_get,
        user_id_handler_get,
    },
    error::ApiError,
};

#[allow(warnings, unused)]
use crate::middleware::timestamp_guard_middleware;

pub fn create_router() -> Router {
    // Routes without middleware
    let public_routes = Router::new()
        .route("/health", get(health_checker_handler));
    
    let protected_middlewares = ServiceBuilder::new();

    #[cfg(not(debug_assertions))]
    let protected_middlewares = protected_middlewares
        .layer(axum::middleware::from_fn(timestamp_guard_middleware));

    let protected_middlewares = protected_middlewares.into_inner();

    // Routes with middleware
    let protected_routes = Router::new()
        .route(
            "/v1/users",
            get(users_handler_get)
        )
        .route(
            "/v1/user/{id}",
            get(user_id_handler_get)
        )
        .layer(
            protected_middlewares
        );

    // Merge routes and add shared state and fallback
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback(|| async { ApiError::NotFound("not found".to_string()).into_response() })
}
