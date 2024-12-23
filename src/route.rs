use axum::{
    middleware,
    routing::{get, post},
    Router,
    response::IntoResponse,
};
use tower::ServiceBuilder;

use crate::{
    handler::{
        create_todo_handler, delete_todo_handler, edit_todo_handler, get_todo_handler,
        health_checker_handler, todos_list_handler,
    },
    middleware::validate_middleware,
    model,
    error::ApiError,
};

pub fn create_router() -> Router {
    let db = model::todo_db();

    // Routes without middleware
    let public_routes = Router::new()
        .route("/api/health", get(health_checker_handler));

    // Routes with middleware
    let protected_routes = Router::new()
        .route(
            "/api/todos",
            post(create_todo_handler).get(todos_list_handler),
        )
        .route(
            "/api/todos/:id",
            get(get_todo_handler)
                .patch(edit_todo_handler)
                .delete(delete_todo_handler),
        )
        .layer(ServiceBuilder::new().layer(middleware::from_fn(validate_middleware)));

    // Merge routes and add shared state and fallback
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback(|| async { ApiError::NotFound.into_response() })
        .with_state(db)
}
