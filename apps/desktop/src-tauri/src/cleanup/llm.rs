use super::prompts::build_system_prompt;
use serde::{Deserialize, Serialize};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const MODEL: &str = "claude-haiku-4-5-20251001";

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

/// Clean up dictated text using Claude Haiku
pub async fn cleanup(
    text: &str,
    language: &str,
    app_context: &str,
    api_key: &str,
) -> Result<String, anyhow::Error> {
    if api_key.is_empty() {
        // If no API key, return text as-is
        log::warn!("No Anthropic API key configured, skipping cleanup");
        return Ok(text.to_string());
    }

    if text.trim().is_empty() {
        return Ok(String::new());
    }

    let system_prompt = build_system_prompt(language, app_context);

    let request = AnthropicRequest {
        model: MODEL.to_string(),
        max_tokens: 8192,
        system: system_prompt,
        messages: vec![Message {
            role: "user".to_string(),
            content: text.to_string(),
        }],
    };

    let client = reqwest::Client::new();
    let response = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Anthropic API error ({}): {}",
            status,
            body
        ));
    }

    let result: AnthropicResponse = response.json().await?;

    let cleaned = result
        .content
        .first()
        .and_then(|block| block.text.clone())
        .unwrap_or_else(|| text.to_string());

    Ok(cleaned)
}
