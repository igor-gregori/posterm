use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::http::models::{KeyValue, Method, RequestModel};

#[derive(Serialize, Deserialize, Clone)]
pub struct SavedRequest {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<SavedKeyValue>,
    pub body: String,
    pub params: Vec<SavedKeyValue>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SavedKeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Collection {
    pub name: String,
    pub requests: Vec<SavedRequest>,
}

impl SavedRequest {
    pub fn from_model(name: &str, model: &RequestModel) -> Self {
        Self {
            name: name.to_string(),
            method: model.method.as_str().to_string(),
            url: model.url.clone(),
            headers: model
                .headers
                .iter()
                .filter(|kv| !kv.key.is_empty())
                .map(|kv| SavedKeyValue {
                    key: kv.key.clone(),
                    value: kv.value.clone(),
                })
                .collect(),
            body: model.body.clone(),
            params: model
                .params
                .iter()
                .filter(|kv| !kv.key.is_empty())
                .map(|kv| SavedKeyValue {
                    key: kv.key.clone(),
                    value: kv.value.clone(),
                })
                .collect(),
        }
    }

    pub fn to_model(&self) -> RequestModel {
        let method = match self.method.as_str() {
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "DELETE" => Method::Delete,
            "PATCH" => Method::Patch,
            _ => Method::Get,
        };
        let mut headers: Vec<KeyValue> = self
            .headers
            .iter()
            .map(|kv| KeyValue {
                key: kv.key.clone(),
                value: kv.value.clone(),
            })
            .collect();
        if headers.is_empty() {
            headers.push(KeyValue {
                key: String::new(),
                value: String::new(),
            });
        }
        let mut params: Vec<KeyValue> = self
            .params
            .iter()
            .map(|kv| KeyValue {
                key: kv.key.clone(),
                value: kv.value.clone(),
            })
            .collect();
        if params.is_empty() {
            params.push(KeyValue {
                key: String::new(),
                value: String::new(),
            });
        }
        RequestModel {
            method,
            url: self.url.clone(),
            headers,
            body: self.body.clone(),
            params,
        }
    }
}

pub fn collections_dir() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("posterm")
        .join("collections");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn load_collections() -> Vec<Collection> {
    let dir = collections_dir();
    let mut collections = Vec::new();

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return collections,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(col) = serde_json::from_str::<Collection>(&content) {
                    collections.push(col);
                }
            }
        }
    }

    collections.sort_by(|a, b| a.name.cmp(&b.name));
    collections
}

pub fn save_collection(collection: &Collection) -> Result<(), String> {
    let dir = collections_dir();
    let filename = format!("{}.json", sanitize_filename(&collection.name));
    let path = dir.join(filename);
    let json = serde_json::to_string_pretty(collection).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn delete_collection(name: &str) -> Result<(), String> {
    let dir = collections_dir();
    let filename = format!("{}.json", sanitize_filename(name));
    let path = dir.join(filename);
    fs::remove_file(&path).map_err(|e| e.to_string())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
