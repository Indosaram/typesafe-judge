use anyhow::{bail, Result};
use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::client::TypeSafeClient;
use crate::formatter::print_pretty_summary;
use crate::input::resolve_state;
use crate::models::{Question, ScoreQuestion, SystemOneRequest, SystemOneResponse};

#[derive(Args, Debug)]
pub struct ScoreArgs {
    #[arg(short, long)]
    pub state: Option<String>,

    #[arg(short, long)]
    pub file: Option<PathBuf>,

    #[arg(short, long)]
    pub instructions: String,

    #[arg(short, long = "level", required = true)]
    pub levels: Vec<String>,

    #[arg(short, long)]
    pub quiet: bool,
}

pub async fn execute(args: ScoreArgs, client: &TypeSafeClient, model: &str, pretty: bool) -> Result<()> {
    if args.levels.len() < 2 {
        bail!("At least 2 levels are required for a score question.");
    }

    let state = resolve_state(args.state, args.file)?;

    let question = Question::Score(ScoreQuestion {
        instructions: serde_json::Value::String(args.instructions),
        criteria: args.levels,
    });

    let mut questions = HashMap::new();
    questions.insert("score".to_string(), question);

    let req = SystemOneRequest {
        state,
        model: model.to_string(),
        questions,
    };

    let (raw_text, elapsed) = client.evaluate_raw(&req).await?;

    if args.quiet {
        let val: serde_json::Value = serde_json::from_str(&raw_text)?;
        if let Some(s) = val["answers"]["score"]["score"].as_f64() {
            println!("{:.2}", s);
        }
        return Ok(());
    }

    if pretty {
        let res: SystemOneResponse = serde_json::from_str(&raw_text)?;
        print_pretty_summary(&res, elapsed);
        return Ok(());
    }

    println!("{}", raw_text);
    Ok(())
}
