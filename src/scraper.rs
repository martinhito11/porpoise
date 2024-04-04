use reqwest;
use serde_json::json;
use serde_json::Value;
use std::error::Error;

async fn search_google(query: &str, api_key: &str, cx: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let url = "https://www.googleapis.com/customsearch/v1/";

    let payload = json!({
        "key": api_key,
        "cx": cx,
        "q": query
    });

    println!("Payload: {}", payload);
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .query(&payload)
        .send()
        .await?;

    // Check if the request was successful
    if resp.status().is_success() {
        let text = resp.text().await?;
        Ok(extract_query_items(&text))
    } else {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Request failed")))
    }
}

fn extract_query_items(html_content: &str) -> Vec<String> {
    let num_results = 2;
    let parsed_response: Result<Value, _> = serde_json::from_str(html_content);

    // Check if parsing was successful
    match parsed_response {
        Ok(parsed_json) => {
            if let Some(items) = parsed_json["items"].as_array() {
                let mut results = Vec::new();
                for item in items.iter().take(num_results) {
                    results.push(item.to_string());
                }
                results
            } else {
                Vec::new()
            }
        }
        Err(err) => {
            eprintln!("Error parsing JSON: {}", err);
            Vec::new()
        }
    }
}

pub async fn get_online_info(query: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let api_key = "AIzaSyATqy-5Vogt_69sZuaI6rg6fN5bV4grqrk";
    let cx = "036e23a64725e4446";

    let query_string = &query;
    match search_google(query_string, api_key, cx).await {
        Ok(paragraphs) => Ok(paragraphs),
        Err(err) => Err(err.into()),
    }
}