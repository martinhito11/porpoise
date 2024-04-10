use ammonia::Builder;
use reqwest;
use regex::Regex;
use serde_json::json;
use serde_json::Value;
use std::error::Error;
use std::collections::HashMap;

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
    let num_results = 1;
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

pub async fn get_clean_site_body(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;

    if response.status().is_success() {
        let body = response.bytes().await?;
        let body_string = String::from_utf8_lossy(&body).to_string();

        Ok(clean_html(&body_string))
    } else {
        eprintln!("Request failed with status code: {}", response.status());
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Request failed")))
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


// helper function to remove all HTML fragments
pub fn clean_html(input: &str) -> String {
    let stripped_html = Builder::default()
        .tags(std::collections::HashSet::new())
        .clean(input)
        .to_string();

    // regex to match contiguous whitespace characters and replace with single space
    let regex = Regex::new(r"\s+").unwrap();
    regex.replace_all(&stripped_html, " ").to_string()
}