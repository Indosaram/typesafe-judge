use anyhow::{anyhow, Context, Result};
use reqwest::{Client, StatusCode};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::time::sleep;

use crate::models::SystemOneRequest;

const DEFAULT_API_URL: &str = "https://api.typesafe.ai/v1/systemone";
const MAX_RETRIES: u32 = 3;

pub struct TypeSafeClient {
    client: Client,
    api_key: String,
    api_url: String,
}

fn find_key_in_env_file(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("TYPESAFE_API_KEY=") {
            let val = rest.trim().trim_matches('"').trim_matches('\'');
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

fn resolve_api_key(explicit_key: Option<String>) -> Result<String> {
    if let Some(k) = explicit_key {
        let trimmed = k.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    if let Ok(k) = std::env::var("TYPESAFE_API_KEY") {
        let trimmed = k.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let mut curr = Some(cwd.as_path());
        while let Some(dir) = curr {
            let env_file = dir.join(".env");
            if env_file.is_file() {
                if let Some(k) = find_key_in_env_file(&env_file) {
                    return Ok(k);
                }
            }
            curr = dir.parent();
        }
    }

    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        let candidates = [
            home.join(".config").join("typesafe").join("api_key"),
            home.join(".typesafe").join("api_key"),
        ];
        for path in candidates {
            if path.is_file() {
                if let Ok(content) = fs::read_to_string(path) {
                    let trimmed = content.trim().to_string();
                    if !trimmed.is_empty() {
                        return Ok(trimmed);
                    }
                }
            }
        }
    }

    Err(anyhow!(
        "TYPESAFE_API_KEY could not be found via --api-key, environment variable, .env, or ~/.config/typesafe/api_key."
    ))
}

impl TypeSafeClient {
    pub fn new(api_key: Option<String>, api_url: Option<String>) -> Result<Self> {
        let key = resolve_api_key(api_key)?;
        let url = api_url.unwrap_or_else(|| DEFAULT_API_URL.to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            api_key: key,
            api_url: url,
        })
    }

    pub async fn evaluate_raw(&self, request: &SystemOneRequest) -> Result<(String, u128)> {
        let start = Instant::now();
        let mut attempts = 0;
        let mut delay = Duration::from_millis(500);

        loop {
            attempts += 1;
            let resp = self
                .client
                .post(&self.api_url)
                .header("Authorization", format!("Bearer {}", self.api_key.trim()))
                .header("Content-Type", "application/json")
                .json(request)
                .send()
                .await
                .context("Failed to connect to TypeSafe API")?;

            let status = resp.status();

            if status == StatusCode::TOO_MANY_REQUESTS || status.as_u16() == 529 {
                if attempts <= MAX_RETRIES {
                    sleep(delay).await;
                    delay *= 2;
                    continue;
                }
            }

            let elapsed = start.elapsed().as_millis();
            let raw_text = resp.text().await.context("Failed to read response body")?;

            if !status.is_success() {
                return Err(anyhow!(
                    "TypeSafe API returned error {}: {}",
                    status,
                    raw_text
                ));
            }

            return Ok((raw_text, elapsed));
        }
    }
}
