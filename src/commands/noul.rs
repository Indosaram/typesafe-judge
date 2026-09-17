use anyhow::Result;
use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::client::TypeSafeClient;
use crate::formatter::print_pretty_summary;
use crate::input::resolve_state;
use crate::models::{NoulCriteria, NoulQuestion, Question, SystemOneRequest, SystemOneResponse};

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

    let (raw_text, elapsed) = client.evaluate_raw(&req).await?;

    let val: serde_json::Value = serde_json::from_str(&raw_text)?;
    let prob = val["answers"]["noul"]["noul"].as_f64().unwrap_or(0.0);
    let passed = prob >= args.threshold;

    if args.quiet {
        println!("{:.3}", prob);
        if !passed {
            std::process::exit(1);
        }
        return Ok(());
    }

    if pretty {
        let res: SystemOneResponse = serde_json::from_str(&raw_text)?;
        print_pretty_summary(&res, elapsed);
        if !passed {
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("{}", raw_text);

    if !passed {
        std::process::exit(1);
    }

    Ok(())
}
