use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::storage::collections::SavedRequest;

const DEFAULT_MAX_ENTRIES: usize = 50;

#[derive(Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub request: SavedRequest,
    pub status: Option<u16>,
    pub duration_ms: Option<u64>,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct History {
    pub max_entries: usize,
    pub entries: Vec<HistoryEntry>,
}

impl History {
    pub fn new() -> Self {
        Self {
            max_entries: DEFAULT_MAX_ENTRIES,
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, entry: HistoryEntry) {
        self.entries.insert(0, entry);
        self.entries.truncate(self.max_entries);
    }
}

pub fn history_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("posterm");
    fs::create_dir_all(&dir).ok();
    dir.join("history.json")
}

pub fn load_history() -> History {
    let path = history_path();
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| History::new()),
        Err(_) => History::new(),
    }
}

pub fn save_history(history: &History) -> Result<(), String> {
    let path = history_path();
    let json = serde_json::to_string_pretty(history).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

/// Create a timestamp string for the current time
pub fn now_timestamp() -> String {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    let secs = duration.as_secs();
    // Simple human-readable format: HH:MM:SS
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
