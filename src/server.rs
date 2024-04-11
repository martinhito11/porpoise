use axum::{
    extract::Json,
    response::{Json as JsonResponse, IntoResponse, Html},
    routing::{get, post},
    Router
};

use std::{
    error::Error, 
    fs, 
    net::SocketAddr, 
    path::Path
};

use serde_json::json;

mod openai;
mod api_dtos;
mod helpers;
mod scraper;
mod serpstack;

use crate::api_dtos::{ChatCompletionRequestMessage, CreateChatCompletionRequest, Role};

async fn index() -> Html<String> {
    println!("Received request to hit index");
    let path = Path::new("pages/index.html");
    let contents = fs::read_to_string(path).unwrap();
    Html(contents)
}

async fn handle_chat_completion(Json(req): Json<CreateChatCompletionRequest>) -> impl IntoResponse {
    println!("Received chat completion request: {:?}", req);
    let n: i32 = 4;

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
    let scraped_pages = scraper::get_online_info(&googleable_query, &n).await;
    if scraped_pages.clone().len() > 0 { println!("got scraped pages: {}", scraped_pages[0].clone()); }
    else { println!("no webpages were found") }
    
    // build new request 
    let mut msg: String = "".to_string();
    for page in scraped_pages.clone() {
        msg.push_str(&page);
        msg.push_str("\n");
    }
    if scraped_pages.len() > 0 {
        msg.push_str(openai::WITH_INFO_USER_QUERY_STR);
    }
    
    msg.push_str(&user_messages[0]);

    println!("final msg: {}", msg.clone());

    let req_message_user: ChatCompletionRequestMessage = ChatCompletionRequestMessage {
        role: Role::User,
        content: msg.to_string()
    };
    let req_with_info: CreateChatCompletionRequest = CreateChatCompletionRequest {
        model: req.model,
        messages: vec![req_message_user]
    };

    // send new request to openai
    let resp = openai::send_chat_completion(req_with_info).await;
    match resp {
        Ok(query) => println!("final message: {}", query.message),
        Err(err) => println!("final message failed: {}", err),
    }
    // If too many tokens, reduce to n-1 page results 

    
    JsonResponse::from(json!({
        "status": "success",
        "message": "Request received successfully"
    }));
}



// async fn test_googler() -> Result<Vec<String>, Box<dyn Error>> {
//     println!("Testing googler");
//     scraper::get_online_info("nyc earthquake", 3).await
// }

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

