use anyhow::{bail, Result};
use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::client::TypeSafeClient;
use crate::formatter::print_pretty_summary;
use crate::input::resolve_state;
use crate::models::{ChoiceQuestion, Question, SystemOneRequest, SystemOneResponse};

#[derive(Args, Debug)]
pub struct ChoiceArgs {
    #[arg(short, long)]
    pub state: Option<String>,

    #[arg(short, long)]
    pub file: Option<PathBuf>,

    #[arg(short, long, default_value = "Choose the most appropriate option based on the context.")]
    pub instructions: String,

    #[arg(short, long = "option", required = true)]
    pub options: Vec<String>,

    #[arg(short, long)]
    pub quiet: bool,
}

pub async fn execute(args: ChoiceArgs, client: &TypeSafeClient, model: &str, pretty: bool) -> Result<()> {
    if args.options.len() < 2 {
        bail!("At least 2 options are required for a choice question.");
    }

    let state = resolve_state(args.state, args.file)?;

    let mut criteria: HashMap<String, Option<String>> = HashMap::new();
    for opt in &args.options {
        if let Some((k, v)) = opt.split_once(':') {
            criteria.insert(k.trim().to_string(), Some(v.trim().to_string()));
        } else {
            criteria.insert(opt.trim().to_string(), None);
        }
    }

    let question = Question::Choice(ChoiceQuestion {
        instructions: serde_json::Value::String(args.instructions),
        criteria,
    });

    let mut questions = HashMap::new();
    questions.insert("choice".to_string(), question);

    let req = SystemOneRequest {
        state,
        model: model.to_string(),
        questions,
    };

    let (raw_text, elapsed) = client.evaluate_raw(&req).await?;

    if args.quiet {
        let val: serde_json::Value = serde_json::from_str(&raw_text)?;
        if let Some(c) = val["answers"]["choice"]["choice"].as_str() {
            println!("{}", c);
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
