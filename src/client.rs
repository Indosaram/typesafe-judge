use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use std::time::Instant;

use crate::models::{SystemOneRequest, SystemOneResponse};

const DEFAULT_API_URL: &str = "https://api.typesafe.ai/v1/systemone";

pub struct TypeSafeClient {
    client: Client,
    api_key: String,
    api_url: String,
}

impl TypeSafeClient {
    pub fn new(api_key: Option<String>, api_url: Option<String>) -> Result<Self> {
        let key = match api_key {
            Some(k) if !k.trim().is_empty() => k,
            _ => std::env::var("TYPESAFE_API_KEY")
                .context("TYPESAFE_API_KEY is not set. Provide --api-key or export TYPESAFE_API_KEY.")?,
        };

        let url = api_url.unwrap_or_else(|| DEFAULT_API_URL.to_string());

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            api_key: key,
            api_url: url,
        })
    }

    pub async fn evaluate(&self, request: &SystemOneRequest) -> Result<(SystemOneResponse, u128)> {
        let start = Instant::now();

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
        let elapsed = start.elapsed().as_millis();

        if !status.is_success() {
            let error_body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "TypeSafe API returned error {}: {}",
                status,
                error_body
            ));
        }

        let body: SystemOneResponse = resp
            .json()
            .await
            .context("Failed to parse TypeSafe API response")?;

        Ok((body, elapsed))
    }
}
