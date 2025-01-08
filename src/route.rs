use axum::{
    middleware,
    routing::{get},
    Router,
    response::IntoResponse,
};
use tower::ServiceBuilder;

use crate::{
    handler::{
        health_checker_handler,
        user_handler_get,
        user_id_handler_get,
        lesson_handler_get
    },
    middleware::{validate_middleware, sync_user_middleware},
    error::ApiError,
};

pub fn create_router() -> Router {
    // Routes without middleware
    let public_routes = Router::new()
        .route("/api/v1/health", get(health_checker_handler));

    // Routes with middleware
    let protected_routes = Router::new()
        .route(
            "/api/v1/user",
            get(user_handler_get)
        )
        .route(
            "/api/v1/user/:id",
            get(user_id_handler_get)
        )
        .route(
            "/api/v1/lesson",
            get(lesson_handler_get)
        )
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(validate_middleware))
                .layer(middleware::from_fn(sync_user_middleware))
        );

    // Merge routes and add shared state and fallback
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback(|| async { ApiError::NotFound("not found".to_string()).into_response() })
}
