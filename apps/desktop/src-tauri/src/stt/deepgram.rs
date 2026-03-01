use serde::Deserialize;

const DEEPGRAM_API_URL: &str = "https://api.deepgram.com/v1/listen";

#[derive(Deserialize)]
struct DeepgramResponse {
    results: DeepgramResults,
}

#[derive(Deserialize)]
struct DeepgramResults {
    channels: Vec<DeepgramChannel>,
}

#[derive(Deserialize)]
struct DeepgramChannel {
    alternatives: Vec<DeepgramAlternative>,
}

#[derive(Deserialize)]
struct DeepgramAlternative {
    transcript: String,
}

/// Transcribe audio using Deepgram Nova-3 API
pub async fn transcribe(
    audio_data: &[u8],
    language: &str,
    api_key: &str,
) -> Result<String, anyhow::Error> {
    if api_key.is_empty() {
        return Err(anyhow::anyhow!(
            "Deepgram API key not configured. Add it in Settings."
        ));
    }

    let lang = match language {
        "pt" | "pt-BR" => "pt-BR",
        "auto" => "auto",
        other => other,
    };

    let url = format!(
        "{}?model=nova-3&language={}&smart_format=true&punctuate=true",
        DEEPGRAM_API_URL, lang
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Token {}", api_key))
        .header("Content-Type", "audio/wav")
        .body(audio_data.to_vec())
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Deepgram API error ({}): {}",
            status,
            body
        ));
    }

    let result: DeepgramResponse = response.json().await?;

    let transcript = result
        .results
        .channels
        .first()
        .and_then(|c| c.alternatives.first())
        .map(|a| a.transcript.clone())
        .unwrap_or_default();

    Ok(transcript)
}
