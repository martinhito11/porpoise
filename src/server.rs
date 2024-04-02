use axum::{
    extract::Json,
    response::{Json as JsonResponse, IntoResponse, Html},
    routing::{get, post},
    Router
};
use std::{fs, path::Path};
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use serde_json::json;

async fn index() -> Html<String> {
    println!("Received request to hit index");
    let path = Path::new("pages/index.html"); // Path to your HTML file
    let contents = fs::read_to_string(path).unwrap();
    Html(contents)
}

async fn handle_chat_completion(Json(req): Json<CreateChatCompletionRequest>) -> impl IntoResponse {
    println!("Received chat completion request: {:?}", req);
    JsonResponse::from(json!({
        "status": "success",
        "message": "Request received successfully"
    }))

}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/", get(index))
        .route("/chat/completions", post(handle_chat_completion));

    let addr = SocketAddr::from(([127, 0, 0, 1], 7878));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Server running on {}", addr);
    
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateChatCompletionRequest {
    model: String,
    messages: Vec<ChatCompletionRequestMessage>,
}

// TODO: OpenAPI supports multiple input types, add error handling for unsupported inputs
#[derive(Debug, Deserialize, Serialize)]
struct ChatCompletionRequestMessage {
    role: String,
    content: String,
}