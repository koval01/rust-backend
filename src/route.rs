use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;

use crate::{
    handler::{
        create_todo_handler, delete_todo_handler, edit_todo_handler, get_todo_handler,
        health_checker_handler, todos_list_handler,
    },
    middleware::validate_middleware,
    model,
};

pub fn create_router() -> Router {
    let db = model::todo_db();

    let app = Router::new()
        .route("/api/health", get(health_checker_handler))
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
        .with_state(db);

    app.layer(ServiceBuilder::new().layer(middleware::from_fn(validate_middleware)))
}
