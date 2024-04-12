use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Html},
    routing::{get, post},
    Router
};

use std::{
    error::Error,
    fs, 
    net::SocketAddr, 
    path::Path
};

mod openai;
mod api_dtos;
mod helpers;
mod scraper;
mod serpstack;

use crate::api_dtos::{ChatCompletionRequest, ChatCompletionRequestMessage, ChatCompletionResponse, Role, ErrorResponse};

async fn index() -> Html<String> {
    println!("Received request to hit index");
    let path = Path::new("pages/index.html");
    let contents = fs::read_to_string(path).unwrap();
    Html(contents)
}

async fn handle_chat_completion(Json(req): Json<ChatCompletionRequest>) -> impl IntoResponse {
    println!("Received chat completion request: {:?}", req);
    let n: i32 = 4;
    let clean_with_openai: bool = true;
    let parallelize: bool = true;
    let use_serpstack: bool = false;

    // parse user query
    let user_messages: Vec<String> = req.messages.iter()
        .filter(|msg| matches!(msg.role, Role::User))
        .map(|msg| msg.content.clone())
        .collect();

    if user_messages.is_empty() {
        panic!("No user messages found.");
    } else if user_messages.len() > 1 {
        panic!("Too many user messages found.")
    }
    else {
        for message in user_messages.clone() {
            println!("User message: {}", message);
        }
    }
    
    // get googleable query
    let googleable_query: String = openai::get_googleable_query(&user_messages[0]).await;    
    println!("got googleable query: {}", &googleable_query);

    // send googleable query to scraper, retrieve cleaned HTML of top n page results 
    let scraped_pages: Vec<String> = if use_serpstack {
        serpstack::get_online_info(&googleable_query).await
    }
    else {
         scraper::get_online_info(&googleable_query, &n, parallelize, clean_with_openai).await
    };
    
    // build new request 
    let mut msg: String = "".to_string();
    for page in scraped_pages.clone() {
        msg.push_str(&page);
        msg.push_str("\n\n");
    }
    if scraped_pages.len() > 0 {
        msg.push_str(openai::WITH_INFO_USER_QUERY_STR);
    }
    msg.push_str(&user_messages[0]);

    println!("final built query: {}", msg.clone());

    let req_message_user: ChatCompletionRequestMessage = ChatCompletionRequestMessage {
        role: Role::User,
        content: msg.to_string()
    };
    let req_with_info: ChatCompletionRequest = ChatCompletionRequest {
        model: req.model,
        messages: vec![req_message_user]
    };

    // send new request to openai
    let resp: Result<ChatCompletionResponse, Box<dyn Error>> = openai::send_chat_completion(req_with_info, false).await; // Assuming this is your async function
    match resp {
        Ok(data) => Json(data).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()).into_response(),
    }
    
    // If too many tokens, reduce to n-1 page results 

    
}

// Helper function to create an error response
fn error_response(status: StatusCode, message: &str) -> impl IntoResponse {
    let error = ErrorResponse {
        error: message.to_string(),
    };
    (status, Json(error))
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

