mod audio;
mod cleanup;
mod hotkey;
mod injection;
mod settings;
mod stt;
mod tray;
mod usage;

use audio::capture::AudioRecorder;
use std::sync::Arc;
use tauri::Manager;
use tauri::tray::TrayIcon;
use tauri_plugin_autostart::MacosLauncher;
use tokio::sync::Mutex;

pub struct AppState {
    pub recorder: Arc<Mutex<AudioRecorder>>,
    /// Lock-free audio level — read without locking the recorder
    pub audio_level: Arc<std::sync::atomic::AtomicU32>,
    pub settings: Arc<Mutex<settings::VozSettings>>,
    pub tray: Arc<std::sync::Mutex<Option<TrayIcon>>>,
    pub usage: Arc<Mutex<usage::UsageStore>>,
}

#[tauri::command]
async fn start_recording(
    state: tauri::State<'_, AppState>,
    language: String,
) -> Result<(), String> {
    let mut recorder = state.recorder.lock().await;
    recorder.start(&language).map_err(|e| e.to_string())?;

    // Update tray icon to recording state
    update_tray(&state, "listening");
    Ok(())
}

#[tauri::command]
async fn stop_recording(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let mut recorder = state.recorder.lock().await;
    let audio_data = recorder.stop().map_err(|e| e.to_string())?;

    // Update tray icon to processing state
    update_tray(&state, "processing");

    let settings = state.settings.lock().await;
    let provider = settings.stt_provider.clone();

    // If user configured multiple languages, use auto-detect for STT
    // to avoid forcing the wrong language (e.g. Whisper biasing toward PT when user speaks EN)
    let stt_language = if settings.personal_languages.len() > 1 {
        log::info!("[stt] Multilingual user ({:?}), using auto-detect", settings.personal_languages);
        "auto".to_string()
    } else {
        log::info!("[stt] Single language, using hint: {}", settings.language);
        settings.language.clone()
    };

    // Estimate audio duration from WAV data: (len - 44 header) / (16000 Hz * 2 bytes per sample)
    let audio_duration_secs = if audio_data.len() > 44 {
        (audio_data.len() - 44) as f64 / (16000.0 * 2.0)
    } else {
        0.0
    };

    // Transcribe based on provider
    let transcript = match provider.as_str() {
        "groq" => {
            stt::groq::transcribe(&audio_data, &stt_language, &settings.groq_api_key)
                .await
                .map_err(|e| e.to_string())?
        }
        "deepgram" => {
            stt::deepgram::transcribe(&audio_data, &stt_language, &settings.deepgram_api_key)
                .await
                .map_err(|e| e.to_string())?
        }
        "assemblyai" => {
            stt::assemblyai::transcribe(&audio_data, &stt_language, &settings.assemblyai_api_key)
                .await
                .map_err(|e| e.to_string())?
        }
        _ => {
            stt::groq::transcribe(&audio_data, &stt_language, &settings.groq_api_key)
                .await
                .map_err(|e| e.to_string())?
        }
    };

    // Track STT usage
    {
        let mut usage = state.usage.lock().await;
        usage.add_stt_usage(&provider, audio_duration_secs);
    }

    // Reset tray to idle
    update_tray(&state, "idle");

    Ok(transcript)
}

#[tauri::command]
async fn get_audio_level(state: tauri::State<'_, AppState>) -> Result<f32, String> {
    // Lock-free read — does NOT lock the recorder, avoiding CoreAudio interference
    let bits = state.audio_level.load(std::sync::atomic::Ordering::Relaxed);
    Ok(f32::from_bits(bits))
}

#[tauri::command]
async fn transcribe_audio(
    state: tauri::State<'_, AppState>,
    audio_path: String,
    language: String,
    provider: String,
) -> Result<String, String> {
    let settings = state.settings.lock().await;
    let audio_data = std::fs::read(&audio_path).map_err(|e| e.to_string())?;

    match provider.as_str() {
        "groq" => stt::groq::transcribe(&audio_data, &language, &settings.groq_api_key)
            .await
            .map_err(|e| e.to_string()),
        "deepgram" => {
            stt::deepgram::transcribe(&audio_data, &language, &settings.deepgram_api_key)
                .await
                .map_err(|e| e.to_string())
        }
        "assemblyai" => {
            stt::assemblyai::transcribe(&audio_data, &language, &settings.assemblyai_api_key)
                .await
                .map_err(|e| e.to_string())
        }
        _ => Err("Unknown provider".to_string()),
    }
}

#[tauri::command]
async fn cleanup_text(
    state: tauri::State<'_, AppState>,
    text: String,
    language: String,
    app_context: String,
) -> Result<String, String> {
    let settings = state.settings.lock().await;
    let api_key = settings.anthropic_api_key.clone();

    // Determine configured languages
    let languages = if settings.personal_languages.is_empty() {
        log::info!("[cleanup] personal_languages empty, using single language: {}", language);
        vec![language]
    } else {
        log::info!("[cleanup] personal_languages: {:?}", settings.personal_languages);
        settings.personal_languages.clone()
    };
    drop(settings); // Release lock before async calls

    log::info!("[cleanup] input text ({} chars): {:?}", text.len(), &text[..text.len().min(100)]);

    // Step 1: If multiple languages configured, detect language first
    let detected_language = if languages.len() > 1 {
        log::info!("[cleanup] Step 1: Detecting language among {:?}...", languages);
        let detect = cleanup::llm::detect_language(&text, &languages, &api_key)
            .await
            .map_err(|e| e.to_string())?;

        log::info!("[cleanup] Detected language: '{}' ({} input, {} output tokens)", detect.language, detect.input_tokens, detect.output_tokens);

        // Track detection usage
        if detect.input_tokens > 0 || detect.output_tokens > 0 {
            let mut usage = state.usage.lock().await;
            usage.add_cleanup_usage(detect.input_tokens, detect.output_tokens);
        }

        detect.language
    } else {
        log::info!("[cleanup] Single language, skipping detection: {}", languages[0]);
        languages[0].clone()
    };

    // Step 2: Cleanup with detected language
    log::info!("[cleanup] Step 2: Cleaning up with language='{}'...", detected_language);
    let result = cleanup::llm::cleanup(&text, &detected_language, &app_context, &api_key)
        .await
        .map_err(|e| e.to_string())?;

    log::info!("[cleanup] Result ({} chars): {:?}", result.text.len(), &result.text[..result.text.len().min(100)]);

    // Track cleanup usage
    if result.input_tokens > 0 || result.output_tokens > 0 {
        let mut usage = state.usage.lock().await;
        usage.add_cleanup_usage(result.input_tokens, result.output_tokens);
    }

    Ok(result.text)
}

#[tauri::command]
async fn inject_text(text: String) -> Result<(), String> {
    injection::paste::inject(&text).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_active_app() -> Result<String, String> {
    injection::accessibility::get_frontmost_app().map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_global_hotkey(_hotkey: String) -> Result<(), String> {
    // Hotkey registration is handled by the plugin
    Ok(())
}

#[tauri::command]
async fn check_permissions() -> Result<serde_json::Value, String> {
    let mic = injection::accessibility::check_microphone_permission();
    let acc = injection::accessibility::check_accessibility_permission();
    Ok(serde_json::json!({
        "microphone": mic,
        "accessibility": acc,
    }))
}

#[tauri::command]
async fn request_microphone_permission() -> Result<bool, String> {
    Ok(injection::accessibility::request_microphone_permission())
}

#[tauri::command]
async fn request_accessibility_permission() -> Result<bool, String> {
    Ok(injection::accessibility::request_accessibility_permission())
}

#[tauri::command]
async fn get_settings(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let settings = state.settings.lock().await;
    serde_json::to_value(&*settings).map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_settings(
    state: tauri::State<'_, AppState>,
    updates: serde_json::Value,
) -> Result<(), String> {
    let mut settings = state.settings.lock().await;

    // Merge updates into current settings
    if let Some(obj) = updates.as_object() {
        if let Some(v) = obj.get("language").and_then(|v| v.as_str()) {
            settings.language = v.to_string();
        }
        if let Some(v) = obj.get("stt_mode").and_then(|v| v.as_str()) {
            settings.stt_mode = v.to_string();
        }
        if let Some(v) = obj.get("stt_provider").and_then(|v| v.as_str()) {
            settings.stt_provider = v.to_string();
        }
        if let Some(v) = obj.get("ai_cleanup").and_then(|v| v.as_bool()) {
            settings.ai_cleanup = v;
        }
        if let Some(v) = obj.get("remove_fillers").and_then(|v| v.as_bool()) {
            settings.remove_fillers = v;
        }
        if let Some(v) = obj.get("fix_grammar").and_then(|v| v.as_bool()) {
            settings.fix_grammar = v;
        }
        if let Some(v) = obj.get("adapt_tone").and_then(|v| v.as_bool()) {
            settings.adapt_tone = v;
        }
        if let Some(v) = obj.get("groq_api_key").and_then(|v| v.as_str()) {
            settings.groq_api_key = v.to_string();
        }
        if let Some(v) = obj.get("deepgram_api_key").and_then(|v| v.as_str()) {
            settings.deepgram_api_key = v.to_string();
        }
        if let Some(v) = obj.get("assemblyai_api_key").and_then(|v| v.as_str()) {
            settings.assemblyai_api_key = v.to_string();
        }
        if let Some(v) = obj.get("anthropic_api_key").and_then(|v| v.as_str()) {
            settings.anthropic_api_key = v.to_string();
        }
        if let Some(v) = obj.get("hotkey").and_then(|v| v.as_str()) {
            settings.hotkey = v.to_string();
        }
        if let Some(v) = obj.get("local_model_path").and_then(|v| v.as_str()) {
            settings.local_model_path = v.to_string();
        }
        if let Some(v) = obj.get("onboarding_complete").and_then(|v| v.as_bool()) {
            settings.onboarding_complete = v;
        }
        if let Some(v) = obj.get("auto_start").and_then(|v| v.as_bool()) {
            settings.auto_start = v;
        }
        if let Some(v) = obj.get("personal_languages").and_then(|v| v.as_array()) {
            settings.personal_languages = v
                .iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect();
        }
    }

    settings.save().map_err(|e| e.to_string())?;
    log::info!("Settings saved to disk");
    Ok(())
}

#[tauri::command]
async fn get_usage_summary(
    state: tauri::State<'_, AppState>,
) -> Result<usage::UsageSummary, String> {
    let usage = state.usage.lock().await;
    Ok(usage.summary())
}

#[tauri::command]
async fn set_auto_start(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| e.to_string())?;
    } else {
        autostart.disable().map_err(|e| e.to_string())?;
    }

    // Persist to settings
    let mut settings = state.settings.lock().await;
    settings.auto_start = enabled;
    settings.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_auto_start(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    let autostart = app.autolaunch();
    autostart.is_enabled().map_err(|e| e.to_string())
}

/// Update tray icon/title based on recording state (called from Rust, no IPC overhead)
fn update_tray(state: &AppState, recording_state: &str) {
    log::info!("[tray] update_tray called with state='{}'", recording_state);
    match state.tray.lock() {
        Ok(tray_guard) => {
            if let Some(tray) = tray_guard.as_ref() {
                let (title, tooltip) = match recording_state {
                    "listening" => ("REC", "Voz — Recording..."),
                    "processing" => ("...", "Voz — Transcribing..."),
                    _ => ("", "Voz — Voice Dictation"),
                };
                let icon = match recording_state {
                    "listening" => tauri::image::Image::from_bytes(
                        include_bytes!("../icons/tray-recording.png"),
                    )
                    .expect("failed to load recording icon"),
                    "processing" => tauri::image::Image::from_bytes(
                        include_bytes!("../icons/tray-processing.png"),
                    )
                    .expect("failed to load processing icon"),
                    _ => tauri::image::Image::from_bytes(
                        include_bytes!("../icons/tray-idle.png"),
                    )
                    .expect("failed to load idle icon"),
                };
                if let Err(e) = tray.set_icon(Some(icon)) {
                    log::error!("[tray] set_icon failed: {}", e);
                }
                // Ensure macOS renders colored icons, not template (monochrome)
                let _ = tray.set_icon_as_template(false);
                if let Err(e) = tray.set_title(Some(title)) {
                    log::error!("[tray] set_title failed: {}", e);
                }
                if let Err(e) = tray.set_tooltip(Some(tooltip)) {
                    log::error!("[tray] set_tooltip failed: {}", e);
                }
                log::info!("[tray] Successfully updated tray to '{}'", recording_state);
            } else {
                log::warn!("[tray] Tray handle is None!");
            }
        }
        Err(e) => {
            log::error!("[tray] Failed to lock tray mutex: {}", e);
        }
    }
}


#[tauri::command]
async fn set_recording_state(
    state: tauri::State<'_, AppState>,
    recording_state: String,
) -> Result<(), String> {
    update_tray(&state, &recording_state);
    Ok(())
}

#[tauri::command]
async fn smart_inject_text(app: tauri::AppHandle, text: String) -> Result<String, String> {
    // Hide the Voz window so the previous app regains focus
    if let Some(win) = app.get_webview_window("main") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
            // Give macOS time to activate the previous app
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }
    }

    let has_field = injection::accessibility::has_focused_text_input();
    if has_field {
        injection::paste::inject(&text).map_err(|e| e.to_string())?;
        Ok("pasted".to_string())
    } else {
        injection::paste::copy_to_clipboard(&text).map_err(|e| e.to_string())?;
        Ok("copied".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let recorder_inner = AudioRecorder::new();
    let audio_level = recorder_inner.level.clone();
    let recorder = Arc::new(Mutex::new(recorder_inner));
    let loaded_settings = settings::VozSettings::load();
    let onboarding_complete = loaded_settings.onboarding_complete;
    let settings = Arc::new(Mutex::new(loaded_settings));
    let tray_handle: Arc<std::sync::Mutex<Option<TrayIcon>>> =
        Arc::new(std::sync::Mutex::new(None));

    let usage = Arc::new(Mutex::new(usage::UsageStore::load()));
    let tray_for_setup = tray_handle.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![])))
        .manage(AppState {
            recorder,
            audio_level,
            settings,
            tray: tray_handle,
            usage,
        })
        .setup(move |app| {
            // Set up tray — store handle
            let tray_icon = tray::setup_tray(app)?;
            if let Ok(mut guard) = tray_for_setup.lock() {
                *guard = Some(tray_icon);
            }

            // Set up global hotkey
            hotkey::setup_hotkey(app)?;

            // Close = hide (quit via tray menu only)
            let main_window = app.get_webview_window("main");
            if let Some(win) = main_window {
                // Show window only if onboarding not complete
                if !onboarding_complete {
                    let _ = win.show();
                    let _ = win.set_focus();
                }

                let win_clone = win.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            log::info!("Voz started successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            get_audio_level,
            transcribe_audio,
            cleanup_text,
            inject_text,
            get_active_app,
            set_global_hotkey,
            check_permissions,
            request_microphone_permission,
            request_accessibility_permission,
            get_settings,
            update_settings,
            set_recording_state,
            smart_inject_text,
            get_usage_summary,
            set_auto_start,
            get_auto_start,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // Show window when user clicks the Dock icon
            if let tauri::RunEvent::Reopen { .. } = event {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
        });
}
