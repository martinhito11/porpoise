use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatCompletionRequestMessage>,
}

// TODO: OpenAPI supports multiple input types, add error handling for unsupported inputs
#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionRequestMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    System,
}
  
#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub system_fingerprint: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: ChatCompletionUsage
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionChoice {
    pub index: i32,
    pub message: ChatCompletionMessage,
    pub logprobs: (),
    pub finish_reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}