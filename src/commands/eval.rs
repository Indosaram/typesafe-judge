use anyhow::{Context, Result};
use clap::Args;
use std::io::{self, Read};
use std::path::PathBuf;

use crate::client::TypeSafeClient;
use crate::formatter::{print_compact_summary, print_pretty_summary};
use crate::models::SystemOneRequest;

#[derive(Args, Debug)]
pub struct EvalArgs {
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    #[arg(long)]
    pub json: bool,
}

pub async fn execute(args: EvalArgs, client: &TypeSafeClient, default_model: &str, pretty: bool) -> Result<()> {
    let raw_json = if let Some(path) = args.file {
        std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read JSON file at {:?}", path))?
    } else {
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_string(&mut buffer)?;
        buffer
    };

    let mut req: SystemOneRequest = serde_json::from_str(&raw_json)
        .context("Failed to parse request JSON for TypeSafe System One")?;

    if req.model.trim().is_empty() {
        req.model = default_model.to_string();
    }

    let (res, elapsed) = client.evaluate(&req).await?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&res)?);
        return Ok(());
    }

    if pretty {
        print_pretty_summary(&res, elapsed);
    } else {
        print_compact_summary(&res, elapsed);
    }

    Ok(())
}
