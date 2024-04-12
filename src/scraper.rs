use ammonia::clean;
use ammonia::Builder;
use reqwest;
use regex::Regex;
use lazy_static::lazy_static;
use serde_json::json;
use serde_json::Value;
use std::error::Error;
use std::collections::HashMap;
use tokio::task;

use crate::helpers;
use crate::api_dtos::{ChatCompletionRequestMessage, CreateChatCompletionRequest, Role};
use crate::openai;

/*
    Parameters: 
        query: string to search 
        n: number of pages to scrape

    Returns: Vector of cleaned HTML body of the top n result URLs 

*/
pub async fn get_online_info(query: &str, n: &i32, parallelize: bool, clean_with_openai: bool) -> Vec<String> {
    let api_key = "AIzaSyATqy-5Vogt_69sZuaI6rg6fN5bV4grqrk";
    let cx = "e04edfd3b386f454b";

    println!("received query for googling: {}", &query);
    // get google custom search API results 
    let google_results = match search_google(&query, api_key, cx, n).await {
        Ok(paragraphs) => paragraphs,
        Err(_) => vec!["".to_string()]
    };

    // get URLs from results 
    let dic_results = helpers::json_vec_to_vec_map(google_results);
    println!("results of google query: {:?}", &dic_results);

    let mut urls: Vec<String> = Vec::new();
    for vec in dic_results {
        let link_value = vec.get("link").unwrap_or_else(|| {
            panic!("No link found in the vector")
        });
    
        let link = link_value.as_str().unwrap_or_else(|| {
            panic!("Link value is not a string")
        });
    
        // check if url in the blacklist 
        let shortened_link = shorten_url(link);
        if !BLACKLISTED_URLS.contains_key(&shortened_link) {
            urls.push(link.to_string())
        }
    }

    // get list of clean HTML bodies of URLs
    let mut clean_bodies = Vec::new();
    if parallelize {
        let futures: Vec<_> = urls.into_iter().take(*n as usize)
            .map(|url| tokio::spawn(get_clean_site_body(url, clean_with_openai)))
            .collect();
    for future in futures {
        if let Ok(result) = future.await {
            clean_bodies.push(result.unwrap_or("".to_string()));
        }
    };
    } else {
        for url in urls.iter() {
            match get_clean_site_body(url.clone().to_string(), clean_with_openai).await {
                Some(clean_body) => 
                    clean_bodies.push(build_url_with_body_str(&url, &clean_body)),
                None => continue
            }
        };
    }

    return clean_bodies;
    
}

fn build_url_with_body_str(url: &str, clean_body: &str) -> String {
    let mut clean_body_with_url = "Source URL: ".to_string();
    clean_body_with_url.push_str(url);
    clean_body_with_url.push_str("\nParagraph:");
    clean_body_with_url.push_str(clean_body); 
    return clean_body_with_url
}

async fn search_google(query: &str, api_key: &str, cx: &str, _n: &i32) -> Result<Vec<String>, Box<dyn Error>> {
    let url = "https://www.googleapis.com/customsearch/v1/";

    let payload = json!({
        "key": api_key,
        "cx": cx,
        "q": query,
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
        println!("response failed, resp: {:?}", resp);
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Google search failed")))
    }
}

fn extract_query_items(html_content: &str) -> Vec<String> {

    let parsed_response: Result<Value, _> = serde_json::from_str(html_content);
    
    // Check if parsing was successful
    match parsed_response {
        Ok(parsed_json) => {
            
            if let Some(items) = parsed_json["items"].as_array() {
                let mut results = Vec::new();
                for item in items.iter() {
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

// helper function to claned up site body for a given url 
async fn get_clean_site_body(url: String, clean_with_openai: bool) -> Option<String> {
    let response = match reqwest::get(url.clone()).await {
        Ok(response) => response,
        Err(_) => return None,
    };

    if response.status().is_success() {
        let body = match response.bytes().await {
            Ok(body) => body,
            Err(_) => return None,
        };
        let body_string = String::from_utf8_lossy(&body).to_string();
        println!("Request to get URL body {} succeeded", url);
        Some(clean_html(&body_string, clean_with_openai).await)
    } else {
        eprintln!("Request to get URL body {} failed with status code: {}", url, response.status());
        None
    }
}

// helper function to remove HTML fragments
async fn clean_html(input: &str, clean_with_openai: bool) -> String {
    let cleaned_html = Builder::default()
        .tags(std::collections::HashSet::new())
        .clean(input)
        .to_string();

    // regex to match contiguous whitespace characters and replace with single space
    let regex = Regex::new(r"\s+").unwrap();
    regex.replace_all(&cleaned_html, " ").to_string();

    // clean with openai if set
    if clean_with_openai {
        println!("found request to clean body with openai");
        let mut clean_query: String = openai::CLEAN_HTML_BODY_QUERY_STR.to_string();
        clean_query.push_str(&cleaned_html);
        let req_message_user: ChatCompletionRequestMessage = ChatCompletionRequestMessage {
            role: Role::User,
            content: clean_query
        };
        let req: CreateChatCompletionRequest = CreateChatCompletionRequest {
            model: openai::DEFAULT_MODEL.to_string(),
            messages: vec![req_message_user]
        };
    
        let resp = openai::send_chat_completion(req).await;
        match resp {
            Ok(query) => query.message,
            Err(_) => cleaned_html,
        }
    }
    else {
        cleaned_html
    }
}

// helper function to remove all parts of a URL after ".com"
// to match it with the URLs in the blacklist 
fn shorten_url(url: &str) -> String {
    if let Some(index) = url.find(".com") {
        let end_index = index + 4;
        println!("shortened to url: {}", String::from(&url[..end_index.clone()]));
        String::from(&url[..end_index])
    } else {
        url.to_string()
    }
}

// hashmap of blacklisted urls to avoid when parsing
// these websites have protections against scraping, and a GET will result in 400 or 999 errors 
lazy_static! {
    static ref BLACKLISTED_URLS: HashMap<String, ()> = {
        let mut m = HashMap::new();
        m.insert("https://www.reddit.com".to_string(), ());
        m.insert("https://www.linkedin.com".to_string(), ());
        m.insert("https://www.quora.com".to_string(), ());
        m.insert("https://www.reuters.com".to_string(), ());
        m.insert("https://www.foodnetwork.com".to_string(), ());
        m
    };
}