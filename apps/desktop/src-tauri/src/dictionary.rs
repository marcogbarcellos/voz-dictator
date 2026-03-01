use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersonalDictionary {
    entries: HashSet<String>,
}

impl PersonalDictionary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, word: String) {
        self.entries.insert(word.to_lowercase());
    }

    pub fn remove(&mut self, word: &str) {
        self.entries.remove(&word.to_lowercase());
    }

    pub fn contains(&self, word: &str) -> bool {
        self.entries.contains(&word.to_lowercase())
    }

    pub fn entries(&self) -> Vec<String> {
        let mut entries: Vec<_> = self.entries.iter().cloned().collect();
        entries.sort();
        entries
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(dict) = serde_json::from_str(&contents) {
                    return dict;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    fn path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("voz")
            .join("dictionary.json")
    }
}
