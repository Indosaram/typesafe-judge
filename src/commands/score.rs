use anyhow::{bail, Result};
use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::client::TypeSafeClient;
use crate::formatter::{print_compact_summary, print_pretty_summary};
use crate::input::resolve_state;
use crate::models::{Answer, Question, ScoreQuestion, SystemOneRequest};

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

    #[arg(long)]
    pub json: bool,
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

    let (res, elapsed) = client.evaluate(&req).await?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&res)?);
        return Ok(());
    }

    if args.quiet {
        if let Some(Answer::Score(score)) = res.answers.get("score") {
            println!("{:.2}", score.score);
        }
        return Ok(());
    }

    if pretty {
        print_pretty_summary(&res, elapsed);
    } else {
        print_compact_summary(&res, elapsed);
    }

    Ok(())
}
