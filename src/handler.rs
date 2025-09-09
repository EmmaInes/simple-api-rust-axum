use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{
    model::{QueryOptions, Todo, UpdateTodoSchema, DB},
    response::{SingleTodoResponse, TodoData, TodoListResponse},
};

#[utoipa::path(
    get,
    path = "/api/healthchecker",
    responses(
        (status = 200, description = "Server is healthy")
    )
)]

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Build Simple CRUD API in Rust using Axum";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

#[utoipa::path(
    get,
    path = "/api/todos",
    responses(
        (status = 200, description = "List all todos", body = TodoListResponse)
    ),
    params(
        ("page" = Option<usize>, Query, description = "Page number"),
        ("limit" = Option<usize>, Query, description = "Items per page")
    )
)]
pub async fn todos_list_handler(
    opts: Option<Query<QueryOptions>>,
    State(db): State<DB>,
) -> impl IntoResponse {
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

    Json(json_response)
}

#[utoipa::path(
    post,
    path = "/api/todos",
    request_body = Todo,
    responses(
        (status = 201, description = "Todo created successfully", body = SingleTodoResponse),
        (status = 409, description = "Todo already exists"),
    )
)]
pub async fn create_todo_handler(
    State(db): State<DB>,
    Json(mut body): Json<Todo>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut vec = db.lock().await;

    if let Some(todo) = vec.iter().find(|todo| todo.title == body.title) {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Todo with title: '{}' already exists", todo.title),
        });
        return Err((StatusCode::CONFLICT, Json(error_response)));
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

#[utoipa::path(
    get,
    path = "/api/todos/{id}",
    responses(
        (status = 200, description = "Get todo by ID", body = SingleTodoResponse),
        (status = 404, description = "Todo not found"),
    ),
    params(
        ("id" = Uuid, Path, description = "Todo ID")
    )
)]

pub async fn get_todo_handler(
    Path(id): Path<Uuid>,
    State(db): State<DB>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let id = id.to_string();
    let vec = db.lock().await;

    if let Some(todo) = vec.iter().find(|todo| todo.id == Some(id.to_owned())) {
        let json_response = SingleTodoResponse {
            status: "success".to_string(),
            data: TodoData { todo: todo.clone() },
        };
        return Ok((StatusCode::OK, Json(json_response)));
    }

    let error_response = serde_json::json!({
        "status": "fail",
        "message": format!("Todo with ID: {} not found", id)
    });
    Err((StatusCode::NOT_FOUND, Json(error_response)))
}

#[utoipa::path(
    patch,
    path = "/api/todos/{id}",
    request_body = UpdateTodoSchema,
    responses(
        (status = 200, description = "Todo updated successfully", body = SingleTodoResponse),
        (status = 404, description = "Todo not found"),
    ),
    params(
        ("id" = Uuid, Path, description = "Todo ID")
    )
)]
pub async fn edit_todo_handler(
    Path(id): Path<Uuid>,
    State(db): State<DB>,
    Json(body): Json<UpdateTodoSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Todo with ID: {} not found", id)
        });

        Err((StatusCode::NOT_FOUND, Json(error_response)))
    }
}

#[utoipa::path(
    delete,
    path = "/api/todos/{id}",
    responses(
        (status = 204, description = "Todo deleted successfully"),
        (status = 404, description = "Todo not found"),
    ),
    params(
        ("id" = Uuid, Path, description = "Todo ID")
    )
)]
pub async fn delete_todo_handler(
    Path(id): Path<Uuid>,
    State(db): State<DB>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let id = id.to_string();
    let mut vec = db.lock().await;

    if let Some(pos) = vec.iter().position(|todo| todo.id == Some(id.clone())) {
        vec.remove(pos);
        return Ok((StatusCode::NO_CONTENT, Json("")));
    }

    let error_response = serde_json::json!({
        "status": "fail",
        "message": format!("Todo with ID: {} not found", id)
    });

    Err((StatusCode::NOT_FOUND, Json(error_response)))
}
