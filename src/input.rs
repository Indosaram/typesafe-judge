use anyhow::{bail, Context, Result};
use std::io::{self, Read};
use std::path::PathBuf;

pub fn resolve_state(
    state_arg: Option<String>,
    file_arg: Option<PathBuf>,
) -> Result<serde_json::Value> {
    let raw_text = if let Some(text) = state_arg {
        text
    } else if let Some(path) = file_arg {
        std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read state file at {:?}", path))?
    } else {
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_string(&mut buffer)?;
        if buffer.trim().is_empty() {
            bail!("No state provided. Supply --state, --file, or pipe content through stdin.");
        }
        buffer
    };

    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&raw_text) {
        if json_val.is_object() || json_val.is_array() {
            return Ok(json_val);
        }
    }

    Ok(serde_json::Value::String(raw_text))
}
