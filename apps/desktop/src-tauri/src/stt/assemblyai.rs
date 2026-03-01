use serde::{Deserialize, Serialize};

const ASSEMBLYAI_UPLOAD_URL: &str = "https://api.assemblyai.com/v2/upload";
const ASSEMBLYAI_TRANSCRIPT_URL: &str = "https://api.assemblyai.com/v2/transcript";

#[derive(Deserialize)]
struct UploadResponse {
    upload_url: String,
}

#[derive(Serialize)]
struct TranscriptRequest {
    audio_url: String,
    language_code: Option<String>,
    language_detection: bool,
}

#[derive(Deserialize)]
struct TranscriptResponse {
    id: String,
    status: String,
    text: Option<String>,
    error: Option<String>,
}

/// Transcribe audio using AssemblyAI Universal-2 API
pub async fn transcribe(
    audio_data: &[u8],
    language: &str,
    api_key: &str,
) -> Result<String, anyhow::Error> {
    if api_key.is_empty() {
        return Err(anyhow::anyhow!(
            "AssemblyAI API key not configured. Add it in Settings."
        ));
    }

    let client = reqwest::Client::new();

    // Step 1: Upload audio
    let upload_resp = client
        .post(ASSEMBLYAI_UPLOAD_URL)
        .header("Authorization", api_key)
        .header("Content-Type", "application/octet-stream")
        .body(audio_data.to_vec())
        .send()
        .await?;

    if !upload_resp.status().is_success() {
        let status = upload_resp.status();
        let body = upload_resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "AssemblyAI upload error ({}): {}",
            status,
            body
        ));
    }

    let upload: UploadResponse = upload_resp.json().await?;

    // Step 2: Request transcription
    let (language_code, language_detection) = match language {
        "pt" | "pt-BR" => (Some("pt".to_string()), false),
        "auto" => (None, true),
        other => (Some(other.to_string()), false),
    };

    let transcript_req = TranscriptRequest {
        audio_url: upload.upload_url,
        language_code,
        language_detection,
    };

    let transcript_resp = client
        .post(ASSEMBLYAI_TRANSCRIPT_URL)
        .header("Authorization", api_key)
        .header("Content-Type", "application/json")
        .json(&transcript_req)
        .send()
        .await?;

    if !transcript_resp.status().is_success() {
        let status = transcript_resp.status();
        let body = transcript_resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "AssemblyAI transcript error ({}): {}",
            status,
            body
        ));
    }

    let transcript: TranscriptResponse = transcript_resp.json().await?;

    // Step 3: Poll until complete
    let poll_url = format!("{}/{}", ASSEMBLYAI_TRANSCRIPT_URL, transcript.id);

    loop {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let poll_resp = client
            .get(&poll_url)
            .header("Authorization", api_key)
            .send()
            .await?;

        let result: TranscriptResponse = poll_resp.json().await?;

        match result.status.as_str() {
            "completed" => {
                return Ok(result.text.unwrap_or_default());
            }
            "error" => {
                return Err(anyhow::anyhow!(
                    "AssemblyAI transcription failed: {}",
                    result.error.unwrap_or_else(|| "Unknown error".to_string())
                ));
            }
            _ => {
                // "queued" or "processing" — keep polling
                continue;
            }
        }
    }
}
