use axum::Router;
use axum::routing::{get, post};
use axum::serve;
use axum::response::Html;
use std::{fs, path::Path};
use std::net::SocketAddr;

async fn index() -> Html<String> {
    let path = Path::new("pages/index.html"); // Path to your HTML file
    let contents = fs::read_to_string(path).unwrap();
    Html(contents)
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/", get(index));

    let addr = SocketAddr::from(([127, 0, 0, 1], 7878));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
    println!("Server running on {}", addr);
}