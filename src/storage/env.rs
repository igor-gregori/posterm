use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Environment {
    pub name: String,
    pub variables: HashMap<String, String>,
    #[serde(default = "default_color")]
    pub color: String,
}

fn default_color() -> String {
    "green".to_string()
}

pub const ENV_COLORS: &[(&str, u8, u8, u8)] = &[
    ("green", 0, 200, 0),
    ("yellow", 200, 200, 0),
    ("red", 200, 50, 50),
    ("blue", 80, 120, 255),
    ("magenta", 200, 80, 200),
    ("cyan", 0, 200, 200),
    ("orange", 255, 140, 0),
    ("white", 200, 200, 200),
];

pub fn color_to_rgb(color: &str) -> (u8, u8, u8) {
    ENV_COLORS.iter()
        .find(|(name, _, _, _)| *name == color)
        .map(|(_, r, g, b)| (*r, *g, *b))
        .unwrap_or((0, 200, 0))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Environments {
    pub active: Option<String>,
    pub environments: Vec<Environment>,
}

impl Environments {
    pub fn new() -> Self {
        Self {
            active: None,
            environments: Vec::new(),
        }
    }

    pub fn active_env(&self) -> Option<&Environment> {
        self.active.as_ref().and_then(|name| {
            self.environments.iter().find(|e| &e.name == name)
        })
    }
}

pub fn environments_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("posterm");
    fs::create_dir_all(&dir).ok();
    dir.join("environments.json")
}

pub fn load_environments() -> Environments {
    let path = environments_path();
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Environments::new()),
        Err(_) => Environments::new(),
    }
}

pub fn save_environments(envs: &Environments) -> Result<(), String> {
    let path = environments_path();
    let json = serde_json::to_string_pretty(envs).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

/// Interpolates {{variable}} placeholders in a string using the active environment
pub fn interpolate(text: &str, env: Option<&Environment>) -> String {
    let Some(env) = env else {
        return text.to_string();
    };

    let mut result = text.to_string();
    for (key, value) in &env.variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}
