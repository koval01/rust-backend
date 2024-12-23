use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json
};
use uuid::Uuid;

use crate::{
    model::{QueryOptions, Todo, UpdateTodoSchema, DB},
    response::{SingleTodoResponse, TodoData, TodoListResponse},
    error::ApiError,
};

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "CRUD API in Rust using Axum";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn todos_list_handler(
    opts: Option<Query<QueryOptions>>,
    State(db): State<DB>,
) -> Result<impl IntoResponse, ApiError> {
    let todos = db.lock().await;

    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let todos: Vec<Todo> = todos.clone().into_iter().skip(offset).take(limit).collect();

    let json_response = TodoListResponse {
        status: "success".to_string(),
        results: todos.len(),
        todos,
    };

    Ok(Json(json_response))
}

pub async fn create_todo_handler(
    State(db): State<DB>,
    Json(mut body): Json<Todo>,
) -> Result<impl IntoResponse, ApiError> {
    let mut vec = db.lock().await;

    if let Some(_todo) = vec.iter().find(|todo| todo.title == body.title) {
        return Err(ApiError::Conflict);
    }

    let uuid_id = Uuid::new_v4();
    let datetime = chrono::Utc::now();

    body.id = Some(uuid_id.to_string());
    body.completed = Some(false);
    body.createdAt = Some(datetime);
    body.updatedAt = Some(datetime);

    let todo = body.to_owned();

    vec.push(body);

    let json_response = SingleTodoResponse {
        status: "success".to_string(),
        data: TodoData { todo },
    };

    Ok((StatusCode::CREATED, Json(json_response)))
}

pub async fn get_todo_handler(
    Path(id): Path<Uuid>,
    State(db): State<DB>,
) -> Result<impl IntoResponse, ApiError> {
    let id = id.to_string();
    let vec = db.lock().await;

    if let Some(todo) = vec.iter().find(|todo| todo.id == Some(id.to_owned())) {
        let json_response = SingleTodoResponse {
            status: "success".to_string(),
            data: TodoData { todo: todo.clone() },
        };
        return Ok((StatusCode::OK, Json(json_response)));
    }

    Err(ApiError::NotFound)
}

pub async fn edit_todo_handler(
    Path(id): Path<Uuid>,
    State(db): State<DB>,
    Json(body): Json<UpdateTodoSchema>,
) -> Result<impl IntoResponse, ApiError> {
    let id = id.to_string();
    let mut vec = db.lock().await;

    if let Some(todo) = vec.iter_mut().find(|todo| todo.id == Some(id.clone())) {
        let datetime = chrono::Utc::now();
        let title = body
            .title
            .to_owned()
            .unwrap_or_else(|| todo.title.to_owned());
        let content = body
            .content
            .to_owned()
            .unwrap_or_else(|| todo.content.to_owned());
        let completed = body.completed.unwrap_or(todo.completed.unwrap());
        let payload = Todo {
            id: todo.id.to_owned(),
            title: if !title.is_empty() {
                title
            } else {
                todo.title.to_owned()
            },
            content: if !content.is_empty() {
                content
            } else {
                todo.content.to_owned()
            },
            completed: Some(completed),
            createdAt: todo.createdAt,
            updatedAt: Some(datetime),
        };
        *todo = payload;

        let json_response = SingleTodoResponse {
            status: "success".to_string(),
            data: TodoData { todo: todo.clone() },
        };
        Ok((StatusCode::OK, Json(json_response)))
    } else {
        Err(ApiError::NotFound)
    }
}

pub async fn delete_todo_handler(
    Path(id): Path<Uuid>,
    State(db): State<DB>,
) -> Result<impl IntoResponse, ApiError> {
    let id = id.to_string();
    let mut vec = db.lock().await;

    if let Some(pos) = vec.iter().position(|todo| todo.id == Some(id.clone())) {
        vec.remove(pos);
        return Ok((StatusCode::NO_CONTENT, Json("")));
    }

    Err(ApiError::NotFound)
}
