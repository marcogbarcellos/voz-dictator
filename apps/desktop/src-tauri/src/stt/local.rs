use anyhow::{anyhow, Context, Result};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub const BUNDLED_MODEL_FILENAME: &str = "ggml-large-v3-turbo-q8_0.bin";

struct LoadedModel {
    path: PathBuf,
    ctx: WhisperContext,
}

static MODEL: OnceLock<Mutex<Option<LoadedModel>>> = OnceLock::new();

fn wav_bytes_to_f32(audio_data: &[u8]) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::new(Cursor::new(audio_data))
        .context("failed to parse WAV data")?;
    let spec = reader.spec();

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => reader
            .samples::<i16>()
            .map(|s| s.map(|v| v as f32 / i16::MAX as f32))
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("failed to read i16 samples")?,
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("failed to read f32 samples")?,
    };

    if spec.channels > 1 {
        let ch = spec.channels as usize;
        Ok(samples
            .chunks(ch)
            .map(|c| c.iter().sum::<f32>() / ch as f32)
            .collect())
    } else {
        Ok(samples)
    }
}

fn run_whisper(model_path: &Path, language: &str, pcm: &[f32]) -> Result<String> {
    let slot = MODEL.get_or_init(|| Mutex::new(None));
    let mut guard = slot.lock().map_err(|_| anyhow!("whisper model mutex poisoned"))?;

    let needs_load = match guard.as_ref() {
        Some(loaded) => loaded.path != model_path,
        None => true,
    };

    if needs_load {
        log::info!("[stt/local] loading whisper model: {}", model_path.display());
        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().ok_or_else(|| anyhow!("invalid model path"))?,
            params,
        )
        .map_err(|e| anyhow!("failed to load whisper model: {:?}", e))?;
        *guard = Some(LoadedModel {
            path: model_path.to_path_buf(),
            ctx,
        });
    }

    let loaded = guard.as_ref().expect("loaded model");
    let mut state = loaded
        .ctx
        .create_state()
        .map_err(|e| anyhow!("failed to create whisper state: {:?}", e))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_print_progress(false);
    params.set_print_special(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_suppress_blank(true);
    params.set_translate(false);

    let lang_opt: Option<&str> = match language {
        "" | "auto" => None,
        "pt" | "pt-BR" => Some("pt"),
        other => Some(other),
    };
    if let Some(l) = lang_opt {
        params.set_language(Some(l));
    }

    state
        .full(params, pcm)
        .map_err(|e| anyhow!("whisper full() failed: {:?}", e))?;

    let n = state
        .full_n_segments()
        .map_err(|e| anyhow!("failed to count segments: {:?}", e))?;
    let mut out = String::new();
    for i in 0..n {
        let seg = state
            .full_get_segment_text(i)
            .map_err(|e| anyhow!("failed to read segment {}: {:?}", i, e))?;
        out.push_str(&seg);
    }

    Ok(out.trim().to_string())
}

/// Transcribe a WAV buffer locally with whisper.cpp. The model file is loaded
/// once and cached in memory for subsequent calls.
pub async fn transcribe(
    audio_data: &[u8],
    language: &str,
    model_path: &Path,
) -> Result<String> {
    if !model_path.exists() {
        return Err(anyhow!(
            "Whisper model not found at {}. Rebuild the app or set local_model_path.",
            model_path.display()
        ));
    }

    let pcm = wav_bytes_to_f32(audio_data)?;
    if pcm.is_empty() {
        return Err(anyhow!("empty audio buffer"));
    }

    let language = language.to_string();
    let model_path = model_path.to_path_buf();

    tokio::task::spawn_blocking(move || run_whisper(&model_path, &language, &pcm))
        .await
        .context("whisper task panicked")?
}
