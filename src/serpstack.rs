use reqwest;
use serde_json::json;

use std::error::Error;


async fn serpstack_search(query: &str, access_key: &str) -> Result<String, Box<dyn Error>> {
    let url = "http://api.serpstack.com/search";

    let payload = json!({
        "access_key": access_key,
        "query": query,
    });

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .query(&payload)
        .send()
        .await?;

        if resp.status().is_success() {
            let text = resp.text().await?;
            println!("sersptack result: {}", text);
            Ok(text)
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Request failed")))
        }
    }

    pub async fn get_online_info(query: &str) -> Vec<String> {
        let api_key = "YOUR API KEY HERE";
    
        let query_string = &query;
        match serpstack_search(query_string, api_key).await {
            Ok(paragraphs) => vec![paragraphs],
            Err(_) => vec!["".to_string()],
        }
    }