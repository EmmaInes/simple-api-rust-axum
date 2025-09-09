mod handler;
mod model;
mod response;
mod route;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use utoipa_swagger_ui::SwaggerUi;
use tower_http::cors::CorsLayer;
use crate::route::create_router;
use crate::response::{GenericResponse, TodoData, SingleTodoResponse, TodoListResponse};
use crate::model::Todo;
use utoipa::OpenApi;

use crate::model::UpdateTodoSchema;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handler::health_checker_handler,
        crate::handler::create_todo_handler,
        crate::handler::todos_list_handler,
        crate::handler::get_todo_handler,
        crate::handler::edit_todo_handler,
        crate::handler::delete_todo_handler,
    ),
    components(
        schemas(
            GenericResponse,
            TodoData,
            SingleTodoResponse,
            TodoListResponse,
            Todo,
            UpdateTodoSchema // <-- Add this line!
        )
    ),
    tags(
        (name = "todo", description = "Todo management endpoints.")
    )
)]
pub struct ApiDoc;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router()
        .layer(cors)
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-doc/openapi.json", ApiDoc::openapi())
        );

    println!("🚀 Server started successfully at http://localhost:8000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}