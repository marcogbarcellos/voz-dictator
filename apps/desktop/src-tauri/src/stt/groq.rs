use reqwest::multipart;
use serde::Deserialize;

const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
const MODEL: &str = "whisper-large-v3-turbo";

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: String,
}

/// Transcribe audio using Groq's Whisper API
pub async fn transcribe(
    audio_data: &[u8],
    language: &str,
    api_key: &str,
) -> Result<String, anyhow::Error> {
    if api_key.is_empty() {
        return Err(anyhow::anyhow!(
            "Groq API key not configured. Add it in Settings."
        ));
    }

    let file_part = multipart::Part::bytes(audio_data.to_vec())
        .file_name("audio.wav")
        .mime_str("audio/wav")?;

    let mut form = multipart::Form::new()
        .part("file", file_part)
        .text("model", MODEL)
        .text("response_format", "json");

    // Add language hint (Whisper uses ISO 639-1 codes)
    let lang_code = match language {
        "pt" | "pt-BR" => "pt",
        "auto" => "", // Let Whisper auto-detect
        other => other,
    };

    if !lang_code.is_empty() {
        form = form.text("language", lang_code.to_string());
    }

    let client = reqwest::Client::new();
    let response = client
        .post(GROQ_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Groq API error ({}): {}",
            status,
            body
        ));
    }

    let result: TranscriptionResponse = response.json().await?;
    Ok(result.text)
}
