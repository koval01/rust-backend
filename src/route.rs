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
        user_handler_post,
        user_handler_put,
        user_id_handler_get
    },
    middleware::validate_middleware,
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
                .post(user_handler_post)
                .put(user_handler_put),
        )
        .route(
            "/api/v1/user/:id",
            get(user_id_handler_get)
        )
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(validate_middleware))
        );

    // Merge routes and add shared state and fallback
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback(|| async { ApiError::NotFound("not found".to_string()).into_response() })
}
