use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct KeyStore {
    #[serde(default)]
    pub keys: HashMap<String, String>,
}

fn config_path() -> Option<PathBuf> {
    crate::paths::keystore_path()
}

pub fn read_store() -> KeyStore {
    let Some(path) = config_path() else { return KeyStore::default() };
    let Ok(content) = fs::read_to_string(&path) else { return KeyStore::default() };
    serde_json::from_str(&content).unwrap_or_default()
}

fn write_store(store: &KeyStore) -> Result<(), String> {
    let path = config_path().ok_or("no HOME")?;
    let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct KeyStatus {
    pub provider: String,
    pub set: bool,
    pub masked: String,
}

#[tauri::command]
pub fn list_api_keys() -> Vec<KeyStatus> {
    let known = ["openai"];
    let store = read_store();
    known
        .iter()
        .map(|p| {
            let v = store.keys.get(*p);
            let set = v.map_or(false, |k| !k.is_empty());
            let masked = match v {
                Some(k) if k.len() > 8 => format!("{}…{}", &k[..4], &k[k.len() - 4..]),
                Some(k) if !k.is_empty() => "•".repeat(k.len().min(8)),
                _ => "".into(),
            };
            KeyStatus { provider: p.to_string(), set, masked }
        })
        .collect()
}

#[tauri::command]
pub fn set_api_key(provider: String, key: String) -> Result<(), String> {
    let mut store = read_store();
    if key.trim().is_empty() {
        store.keys.remove(&provider);
    } else {
        store.keys.insert(provider, key.trim().to_string());
    }
    write_store(&store)
}

pub fn get_key(provider: &str) -> Option<String> {
    read_store().keys.get(provider).cloned().filter(|k| !k.is_empty())
}
