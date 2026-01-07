//! Ollama integration for local LLM inference
//!
//! This module provides integration with Ollama for local AI capabilities.

use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

/// Default Ollama server URL
const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

/// Ollama client state
pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: DEFAULT_OLLAMA_URL.to_string(),
        }
    }

    pub fn with_url(url: String) -> Self {
        Self {
            client: Client::new(),
            base_url: url,
        }
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

// Global client instance
lazy_static::lazy_static! {
    static ref OLLAMA_CLIENT: Arc<Mutex<OllamaClient>> = Arc::new(Mutex::new(OllamaClient::new()));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatResponse {
    message: ChatMessage,
    done: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct StreamChunk {
    message: Option<StreamMessage>,
    done: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct StreamMessage {
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIChatResponse {
    pub content: String,
    pub done: bool,
    pub error: Option<String>,
}

/// Check if Ollama is available
#[tauri::command]
pub async fn check_ollama_status() -> Result<bool, String> {
    let client = OLLAMA_CLIENT.lock().await;
    let url = format!("{}/api/tags", client.base_url);

    match client.client.get(&url).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

/// List available Ollama models
#[tauri::command]
pub async fn list_ollama_models() -> Result<Vec<OllamaModel>, String> {
    let client = OLLAMA_CLIENT.lock().await;
    let url = format!("{}/api/tags", client.base_url);

    let response = client
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama returned error: {}", response.status()));
    }

    let models: OllamaModelsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(models.models)
}

/// Chat with Ollama (non-streaming)
#[tauri::command]
pub async fn chat_with_ollama(
    model: String,
    messages: Vec<ChatMessage>,
) -> Result<AIChatResponse, String> {
    let client = OLLAMA_CLIENT.lock().await;
    let url = format!("{}/api/chat", client.base_url);

    let request = ChatRequest {
        model,
        messages,
        stream: false,
    };

    let response = client
        .client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

    if !response.status().is_success() {
        return Ok(AIChatResponse {
            content: String::new(),
            done: true,
            error: Some(format!("Ollama returned error: {}", response.status())),
        });
    }

    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(AIChatResponse {
        content: chat_response.message.content,
        done: chat_response.done,
        error: None,
    })
}

/// Chat with Ollama (streaming) - returns chunks via events
#[tauri::command]
pub async fn chat_with_ollama_stream(
    app: tauri::AppHandle,
    model: String,
    messages: Vec<ChatMessage>,
    request_id: String,
) -> Result<(), String> {
    let client = OLLAMA_CLIENT.lock().await;
    let url = format!("{}/api/chat", client.base_url);

    let request = ChatRequest {
        model,
        messages,
        stream: true,
    };

    let response = client
        .client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

    if !response.status().is_success() {
        app.emit(
            &format!("ollama-stream-{}", request_id),
            AIChatResponse {
                content: String::new(),
                done: true,
                error: Some(format!("Ollama returned error: {}", response.status())),
            },
        )
        .ok();
        return Ok(());
    }

    // Process stream
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                buffer.push_str(&String::from_utf8_lossy(&bytes));

                // Process complete JSON lines
                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.trim().is_empty() {
                        continue;
                    }

                    if let Ok(chunk) = serde_json::from_str::<StreamChunk>(&line) {
                        let content = chunk
                            .message
                            .map(|m| m.content)
                            .unwrap_or_default();

                        app.emit(
                            &format!("ollama-stream-{}", request_id),
                            AIChatResponse {
                                content,
                                done: chunk.done,
                                error: None,
                            },
                        )
                        .ok();
                    }
                }
            }
            Err(e) => {
                app.emit(
                    &format!("ollama-stream-{}", request_id),
                    AIChatResponse {
                        content: String::new(),
                        done: true,
                        error: Some(format!("Stream error: {}", e)),
                    },
                )
                .ok();
                break;
            }
        }
    }

    Ok(())
}

/// Generate text completion with Ollama
#[tauri::command]
pub async fn generate_completion(
    model: String,
    prompt: String,
    system_prompt: Option<String>,
) -> Result<AIChatResponse, String> {
    let mut messages = Vec::new();

    if let Some(sys) = system_prompt {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: sys,
        });
    }

    messages.push(ChatMessage {
        role: "user".to_string(),
        content: prompt,
    });

    chat_with_ollama(model, messages).await
}

/// Suggest improvements for document content
#[tauri::command]
pub async fn suggest_improvements(
    model: String,
    content: String,
    suggestion_type: String,
) -> Result<AIChatResponse, String> {
    let system_prompt = match suggestion_type.as_str() {
        "grammar" => "You are a professional editor. Review the following text and suggest grammar and spelling corrections. Be concise and specific.",
        "style" => "You are a writing coach. Review the following text and suggest style improvements to make it more professional and clear. Be concise and specific.",
        "rfp" => "You are an RFP response expert. Review the following text and suggest improvements to make it more compelling and compliant. Be concise and specific.",
        _ => "You are a helpful writing assistant. Review the following text and suggest improvements.",
    };

    generate_completion(
        model,
        format!("Please review and suggest improvements for:\n\n{}", content),
        Some(system_prompt.to_string()),
    )
    .await
}

/// Analyze RFP requirements
#[tauri::command]
pub async fn analyze_rfp(
    model: String,
    rfp_content: String,
) -> Result<AIChatResponse, String> {
    let system_prompt = r#"You are an RFP analysis expert. Analyze the following RFP content and extract:
1. Key requirements (numbered list)
2. Deadlines and milestones
3. Evaluation criteria
4. Critical compliance items
5. Potential risks or concerns

Format your response clearly with headers for each section."#;

    generate_completion(
        model,
        rfp_content,
        Some(system_prompt.to_string()),
    )
    .await
}

/// Set Ollama server URL
#[tauri::command]
pub async fn set_ollama_url(url: String) -> Result<(), String> {
    let mut client = OLLAMA_CLIENT.lock().await;
    client.base_url = url;
    Ok(())
}
