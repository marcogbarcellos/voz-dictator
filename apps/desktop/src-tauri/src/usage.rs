use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Per-minute pricing constants
const GROQ_WHISPER_PER_MIN: f64 = 0.111 / 60.0; // $0.111/hour
const DEEPGRAM_NOVA3_PER_MIN: f64 = 0.0077; // $0.0077/minute (Nova-3 mono PAYG)
const ASSEMBLYAI_PER_MIN: f64 = 0.15 / 60.0; // $0.15/hour (Universal-2)
const CLAUDE_HAIKU_INPUT_PER_MTOK: f64 = 1.00; // $1.00/MTok
const CLAUDE_HAIKU_OUTPUT_PER_MTOK: f64 = 5.00; // $5.00/MTok

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub timestamp: u64, // epoch ms
    pub provider: String,
    pub kind: String, // "stt" or "cleanup"
    #[serde(default)]
    pub audio_duration_secs: f64,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub total_cost: f64,
    pub today_cost: f64,
    pub by_provider: Vec<ProviderCost>,
    pub total_calls: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCost {
    pub provider: String,
    pub cost: f64,
    pub calls: usize,
}

#[derive(Debug)]
pub struct UsageStore {
    records: Vec<UsageRecord>,
}

impl UsageStore {
    pub fn load() -> Self {
        let path = Self::store_path();
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(records) = serde_json::from_str::<Vec<UsageRecord>>(&contents) {
                    return Self { records };
                }
            }
        }
        Self {
            records: Vec::new(),
        }
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let path = Self::store_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.records)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn add_stt_usage(&mut self, provider: &str, audio_duration_secs: f64) {
        let duration_min = audio_duration_secs / 60.0;
        let cost = match provider {
            "groq" => duration_min * GROQ_WHISPER_PER_MIN,
            "deepgram" => duration_min * DEEPGRAM_NOVA3_PER_MIN,
            "assemblyai" => duration_min * ASSEMBLYAI_PER_MIN,
            _ => duration_min * GROQ_WHISPER_PER_MIN, // default to groq
        };

        self.records.push(UsageRecord {
            timestamp: now_ms(),
            provider: provider.to_string(),
            kind: "stt".to_string(),
            audio_duration_secs,
            input_tokens: 0,
            output_tokens: 0,
            estimated_cost_usd: cost,
        });

        let _ = self.save();
    }

    pub fn add_cleanup_usage(&mut self, input_tokens: u64, output_tokens: u64) {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * CLAUDE_HAIKU_INPUT_PER_MTOK;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * CLAUDE_HAIKU_OUTPUT_PER_MTOK;
        let cost = input_cost + output_cost;

        self.records.push(UsageRecord {
            timestamp: now_ms(),
            provider: "anthropic".to_string(),
            kind: "cleanup".to_string(),
            audio_duration_secs: 0.0,
            input_tokens,
            output_tokens,
            estimated_cost_usd: cost,
        });

        let _ = self.save();
    }

    pub fn summary(&self) -> UsageSummary {
        let total_cost: f64 = self.records.iter().map(|r| r.estimated_cost_usd).sum();
        let today_start = today_start_ms();
        let today_cost: f64 = self
            .records
            .iter()
            .filter(|r| r.timestamp >= today_start)
            .map(|r| r.estimated_cost_usd)
            .sum();

        // Group by provider
        let mut provider_map: std::collections::HashMap<String, (f64, usize)> =
            std::collections::HashMap::new();
        for r in &self.records {
            let entry = provider_map.entry(r.provider.clone()).or_insert((0.0, 0));
            entry.0 += r.estimated_cost_usd;
            entry.1 += 1;
        }

        let by_provider: Vec<ProviderCost> = provider_map
            .into_iter()
            .map(|(provider, (cost, calls))| ProviderCost {
                provider,
                cost,
                calls,
            })
            .collect();

        UsageSummary {
            total_cost,
            today_cost,
            by_provider,
            total_calls: self.records.len(),
        }
    }

    fn store_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("voz")
            .join("usage.json")
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn today_start_ms() -> u64 {
    let now = now_ms();
    // Truncate to start of day (UTC)
    let day_ms: u64 = 24 * 60 * 60 * 1000;
    (now / day_ms) * day_ms
}
