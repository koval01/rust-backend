use axum::{
    routing::{get},
    Router,
    response::IntoResponse,
};
use axum::routing::post;
use tower::ServiceBuilder;

use crate::{
    handler::{
        health_checker_handler,
        login, callback,
        user_handler_get,
        user_id_handler_get,
        lesson_handler_get
    },
    error::ApiError,
};

#[allow(warnings, unused)]
use crate::middleware::timestamp_guard_middleware;

pub fn create_router() -> Router {
    // Routes without middleware
    let public_routes = Router::new()
        .route("/api/v1/health", get(health_checker_handler))
        .route("/api/v1/auth/login", get(login))
        .route("/api/v1/auth/callback", get(callback));
    
    let protected_middlewares = ServiceBuilder::new();

    #[cfg(not(debug_assertions))]
    let protected_middlewares = protected_middlewares
        .layer(axum::middleware::from_fn(timestamp_guard_middleware));

    let protected_middlewares = protected_middlewares.into_inner();

    // Routes with middleware
    let protected_routes = Router::new()
        .route(
            "/api/v1/user",
            get(user_handler_get)
        )
        .route(
            "/api/v1/user/{id}",
            get(user_id_handler_get)
        )
        .route(
            "/api/v1/lesson",
            get(lesson_handler_get)
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
