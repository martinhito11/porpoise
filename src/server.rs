use axum::{
    extract::Json,
    response::{Json as JsonResponse, IntoResponse, Html},
    routing::{get, post},
    Router
};
use reqwest;
use std::{
    error::Error, 
    fs, 
    net::SocketAddr, 
    path::Path
};
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
    send_to_openai(req).await;
    JsonResponse::from(json!({
        "status": "success",
        "message": "Request received successfully"
    }))

}

async fn send_to_openai(req: CreateChatCompletionRequest) -> Result<(), Box<dyn Error>> {
    println!("Received request to send to OpenAI");
    let API_KEY = "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";
    let model = "gpt-3.5-turbo";
    
    let messages = vec![
        ChatCompletionRequestMessage {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        },
        ChatCompletionRequestMessage {
            role: "user".to_string(),
            content: "Hello!".to_string(),
        },
    ];
    let payload = CreateChatCompletionRequest { model: model.to_string(), messages };

    // Send the request
    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", API_KEY.to_string())) 
        .body(serde_json::to_string(&payload)?)
        .send()
        .await?;

    // Check if the request was successful
    if resp.status().is_success() {
        let text = resp.text().await?;
        println!("Response: {}", text);
        Ok(())
    } else {
        println!("Request failed with status code: {}", resp.status());
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Request failed")))
    }
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