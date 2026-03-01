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
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct AnthropicUsage {
    input_tokens: u64,
    output_tokens: u64,
}

/// Result of a cleanup call, including token usage for cost tracking
pub struct CleanupResult {
    pub text: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Result of language detection
pub struct DetectResult {
    pub language: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Detect the language of transcribed text from the user's configured languages.
/// Uses a minimal prompt (~50 tokens) to keep costs negligible.
/// Returns the detected language code and token usage.
pub async fn detect_language(
    text: &str,
    configured_languages: &[String],
    api_key: &str,
) -> Result<DetectResult, anyhow::Error> {
    // Build language options: "Portuguese (pt), English (en)"
    let options: Vec<String> = configured_languages
        .iter()
        .map(|code| {
            let name = lang_name(code);
            format!("{} ({})", name, code)
        })
        .collect();

    let codes: Vec<&str> = configured_languages.iter().map(|s| s.as_str()).collect();

    // Take first ~200 chars for detection (roughly 2 sentences)
    let sample: String = text.chars().take(200).collect();

    let system = format!(
        "Identify which language this text is written in. Options: {}. Reply with ONLY the language code ({}). Nothing else.",
        options.join(", "),
        codes.join(", "),
    );

    let request = AnthropicRequest {
        model: MODEL.to_string(),
        max_tokens: 8,
        system,
        messages: vec![Message {
            role: "user".to_string(),
            content: sample,
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
        // On error, fall back to first configured language
        log::warn!("Language detection failed, falling back to {}", configured_languages[0]);
        return Ok(DetectResult {
            language: configured_languages[0].clone(),
            input_tokens: 0,
            output_tokens: 0,
        });
    }

    let result: AnthropicResponse = response.json().await?;

    let detected = result
        .content
        .first()
        .and_then(|block| block.text.clone())
        .unwrap_or_default()
        .trim()
        .to_lowercase();

    let (input_tokens, output_tokens) = match result.usage {
        Some(u) => (u.input_tokens, u.output_tokens),
        None => (0, 0),
    };

    // Validate detected language is one of the configured ones
    let language = if configured_languages.contains(&detected) {
        detected
    } else {
        log::warn!(
            "Detected '{}' not in configured languages {:?}, falling back to {}",
            detected,
            configured_languages,
            configured_languages[0]
        );
        configured_languages[0].clone()
    };

    Ok(DetectResult {
        language,
        input_tokens,
        output_tokens,
    })
}

/// Clean up dictated text using Claude Haiku
pub async fn cleanup(
    text: &str,
    language: &str,
    app_context: &str,
    api_key: &str,
) -> Result<CleanupResult, anyhow::Error> {
    if api_key.is_empty() {
        log::warn!("No Anthropic API key configured, skipping cleanup");
        return Ok(CleanupResult {
            text: text.to_string(),
            input_tokens: 0,
            output_tokens: 0,
        });
    }

    if text.trim().is_empty() {
        return Ok(CleanupResult {
            text: String::new(),
            input_tokens: 0,
            output_tokens: 0,
        });
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

    let (input_tokens, output_tokens) = match result.usage {
        Some(u) => (u.input_tokens, u.output_tokens),
        None => (0, 0),
    };

    Ok(CleanupResult {
        text: cleaned,
        input_tokens,
        output_tokens,
    })
}

fn lang_name(code: &str) -> &str {
    match code {
        "pt" | "pt-BR" => "Portuguese",
        "en" => "English",
        "es" => "Spanish",
        "fr" => "French",
        "de" => "German",
        "it" => "Italian",
        "ja" => "Japanese",
        "ko" => "Korean",
        "zh" => "Chinese",
        _ => code,
    }
}
