use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VozSettings {
    pub language: String,
    pub stt_mode: String,
    pub stt_provider: String,
    pub ai_cleanup: bool,
    pub remove_fillers: bool,
    pub fix_grammar: bool,
    pub adapt_tone: bool,
    pub groq_api_key: String,
    pub deepgram_api_key: String,
    pub assemblyai_api_key: String,
    pub anthropic_api_key: String,
    pub local_model_path: String,
    pub hotkey: String,
    pub personal_dictionary: Vec<String>,
    #[serde(default)]
    pub onboarding_complete: bool,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default)]
    pub personal_languages: Vec<String>,
}

impl Default for VozSettings {
    fn default() -> Self {
        Self {
            language: "pt".to_string(),
            stt_mode: "local".to_string(),
            stt_provider: "local".to_string(),
            ai_cleanup: true,
            remove_fillers: true,
            fix_grammar: true,
            adapt_tone: false,
            groq_api_key: String::new(),
            deepgram_api_key: String::new(),
            assemblyai_api_key: String::new(),
            anthropic_api_key: String::new(),
            local_model_path: String::new(),
            hotkey: "Alt+Space".to_string(),
            personal_dictionary: Vec::new(),
            onboarding_complete: false,
            auto_start: false,
            personal_languages: Vec::new(),
        }
    }
}

impl VozSettings {
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => match serde_json::from_str(&contents) {
                    Ok(settings) => return settings,
                    Err(e) => log::warn!("Failed to parse settings: {}", e),
                },
                Err(e) => log::warn!("Failed to read settings: {}", e),
            }
        }

        Self::default()
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let config_path = Self::config_path();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, json)?;
        Ok(())
    }

    fn config_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("voz")
            .join("settings.json")
    }
}
