use std::fs;
use std::path::Path;
use serde::{Serialize, de::DeserializeOwned};

pub fn load_json<T: DeserializeOwned>(path: &str) -> anyhow::Result<Option<T>> {
    if !Path::new(path).exists() { return Ok(None); }
    let data = fs::read_to_string(path)?;
    let v = serde_json::from_str::<T>(&data)?;
    Ok(Some(v))
}

pub fn save_json<T: Serialize>(path: &str, value: &T) -> anyhow::Result<()> {
    let s = serde_json::to_string_pretty(value)?;
    fs::write(path, s)?;
    Ok(())
}
