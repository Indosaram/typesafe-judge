use anyhow::{bail, Result};
use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::client::TypeSafeClient;
use crate::formatter::print_pretty_summary;
use crate::input::resolve_state;
use crate::models::{Answer, NoulCriteria, NoulQuestion, Question, SystemOneRequest};

#[derive(Args, Debug)]
pub struct NoulArgs {
    #[arg(short, long)]
    pub state: Option<String>,

    #[arg(short, long)]
    pub file: Option<PathBuf>,

    #[arg(short, long)]
    pub instructions: String,

    #[arg(long)]
    pub true_desc: Option<String>,

    #[arg(long)]
    pub false_desc: Option<String>,

    #[arg(short, long, default_value_t = 0.5)]
    pub threshold: f64,

    #[arg(short, long)]
    pub quiet: bool,

    #[arg(long)]
    pub compact: bool,
}

pub async fn execute(args: NoulArgs, client: &TypeSafeClient, model: &str, pretty: bool) -> Result<()> {
    let state = resolve_state(args.state, args.file)?;

    let criteria = if args.true_desc.is_some() || args.false_desc.is_some() {
        Some(NoulCriteria {
            true_desc: args.true_desc,
            false_desc: args.false_desc,
        })
    } else {
        None
    };

    let question = Question::Noul(NoulQuestion {
        instructions: serde_json::Value::String(args.instructions),
        criteria,
    });

    let mut questions = HashMap::new();
    questions.insert("noul".to_string(), question);

    let req = SystemOneRequest {
        state,
        model: model.to_string(),
        questions,
    };

    let (res, elapsed) = client.evaluate(&req).await?;

    let prob = if let Some(Answer::Noul(noul)) = res.answers.get("noul") {
        noul.noul
    } else {
        bail!("Missing noul answer in response");
    };

    let passed = prob >= args.threshold;

    if args.quiet {
        println!("{:.3}", prob);
        if !passed {
            std::process::exit(1);
        }
        return Ok(());
    }

    if pretty {
        print_pretty_summary(&res, elapsed);
        if !passed {
            std::process::exit(1);
        }
        return Ok(());
    }

    if args.compact {
        println!("{}", serde_json::to_string(&res)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&res)?);
    }

    if !passed {
        std::process::exit(1);
    }

    Ok(())
}
