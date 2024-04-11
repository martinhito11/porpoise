use ammonia::Builder;
use reqwest;
use regex::Regex;
use serde_json::json;
use serde_json::Value;
use std::error::Error;

use crate::helpers;

/*
    Parameters: 
        query: string to search 
        n: number of pages to scrape

    Returns: Vector of cleaned HTML body of the top n result URLs 

*/
pub async fn get_online_info(query: &str, n: &i32) -> Vec<String> {
    let api_key = "AIzaSyATqy-5Vogt_69sZuaI6rg6fN5bV4grqrk";
    let cx = "036e23a64725e4446";

    let query_string = &query;
    // get google custom search API results 
    let google_results = match search_google(query_string, api_key, cx, n).await {
        Ok(paragraphs) => paragraphs,
        Err(err_) => vec!["".to_string()]
    };

    // get URLs from results 
    let dic_results = helpers::json_vec_to_vec_map(google_results);
    let mut urls = Vec::new();
    for vec in &dic_results {
        let link_value = vec.get("link").unwrap_or_else(|| {
            panic!("No link found in the vector");
        });
        let link = link_value.as_str().unwrap_or_else(|| {
            panic!("Link value is not a string");
        });
        urls.push(link);
    
    };

    // get list of clean HTML bodies of URLs
    let mut clean_bodies = Vec::new();
    for url in &urls {
        match get_clean_site_body(url).await {
            Some(clean_body) => clean_bodies.push(clean_body),
            None => continue
        }
    };

    return clean_bodies;
    
}

async fn search_google(query: &str, api_key: &str, cx: &str, n: &i32) -> Result<Vec<String>, Box<dyn Error>> {
    let url = "https://www.googleapis.com/customsearch/v1/";

    let payload = json!({
        "key": api_key,
        "cx": cx,
        "q": query,
        "num": n,
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
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Google search failed")))
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

pub async fn get_clean_site_body(url: &str) -> Option<String> {
    let response = match reqwest::get(url).await {
        Ok(response) => response,
        Err(_) => return None,
    };

    if response.status().is_success() {
        let body = match response.bytes().await {
            Ok(body) => body,
            Err(_) => return None,
        };
        let body_string = String::from_utf8_lossy(&body).to_string();

        Some(clean_html(&body_string))
    } else {
        eprintln!("Request to get URL body {} failed with status code: {}", url, response.status());
        None
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