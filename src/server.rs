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
    path::Path,
    collections::HashMap
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

mod scraper;
mod serpstack;

async fn index() -> Html<String> {
    println!("Received request to hit index");
    let path = Path::new("pages/index.html");
    let contents = fs::read_to_string(path).unwrap();
    Html(contents)
}

async fn handle_chat_completion(Json(req): Json<CreateChatCompletionRequest>) -> impl IntoResponse {
    println!("Received chat completion request: {:?}", req);

    send_to_openai(req).await;
    let google_results = match test_googler().await {
        Ok(results) => results,
        Err(err) => {
            eprintln!("Error getting Google results: {:?}", err);
            Vec::new()
        }
    };

    let dic_results = json_vec_to_vec_map(google_results);
    for vec in &dic_results {
        // for (key, value) in vec {
        //     println!("\n{}: {}", key, value);
        // }
        let link_value = vec.get("link").unwrap_or_else(|| {
            panic!("No link found in the vector");
        });
        
        let link = link_value.as_str().unwrap_or_else(|| {
            panic!("Link value is not a string");
        });
        
        let clean_body = scraper::get_clean_site_body(link).await;
        match clean_body {
            Ok(results) => println!("\nresults: {}", results),
            Err(err) => {
                eprintln!("Error getting Google results: {:?}", err);
            }
        }
    }
    
    JsonResponse::from(json!({
        "status": "success",
        "message": "Request received successfully"
    }));
}

async fn test_googler() -> Result<Vec<String>, Box<dyn Error>> {
    println!("Testing googler");
    match scraper::get_online_info("nyc earthquake").await {
        Ok(google) => {
            Ok(google)
        }
        Err(err) => {
            Err(err.into())
        }
    }
}

async fn test_serpstack() -> Result<Vec<String>, Box<dyn Error>> {
    println!("Testing serpstack");
    match serpstack::get_online_info("martin hito").await {
        Ok(google) => {
            Ok(google)
        }
        Err(err) => {
            Err(err.into())
        }
    }
}

fn json_vec_to_vec_map(json_vec: Vec<String>) -> Vec<HashMap<String, Value>> {
    let mut vec_map = Vec::new();
        
    for json_str in json_vec {
        let mut map = HashMap::new();
        if let Ok(parsed_json) = serde_json::from_str::<Value>(&json_str) {
            if let Some(obj) = parsed_json.as_object() {
                for (key, value) in obj {
                    map.insert(key.clone(), value.clone());
                }
            }
            vec_map.push(map);
        }
    }
    vec_map
}

async fn send_to_openai(req: CreateChatCompletionRequest) -> Result<String, Box<dyn Error>> {
    println!("Received request to send to OpenAI");
    let api_key = "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";
    let url = "https://api.openai.com/v1/chat/completions";
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
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key.to_string())) 
        .body(serde_json::to_string(&payload)?)
        .send()
        .await?;

    // Check if the request was successful
    if resp.status().is_success() {
        let text = resp.text().await?;
        println!("Response: {}", text);
        Ok(text)
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

    let addr = SocketAddr::from(([127, 0, 0, 1], 8787));
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