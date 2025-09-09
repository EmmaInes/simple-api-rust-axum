use crate::model::Todo;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, Debug, ToSchema)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct TodoData {
    pub todo: Todo,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct SingleTodoResponse {
    pub status: String,
    pub data: TodoData,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct TodoListResponse {
    pub status: String,
    pub results: usize,
    pub todos: Vec<Todo>,
}
