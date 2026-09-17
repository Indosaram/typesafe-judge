use anyhow::{bail, Context, Result};
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::io::{self, Read, IsTerminal};
use std::process::Command;

use crate::client::TypeSafeClient;
use crate::formatter::print_response_summary;
use crate::models::{
    Answer, NoulCriteria, NoulQuestion, Question, ScoreQuestion, SystemOneRequest,
};

#[derive(Args, Debug)]
pub struct DiffArgs {
    #[arg(short, long)]
    pub prompt: String,

    #[arg(long)]
    pub staged: bool,

    #[arg(short, long)]
    pub quiet: bool,

    #[arg(long)]
    pub json: bool,
}

fn get_git_diff(staged: bool) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.arg("diff");
    if staged {
        cmd.arg("--staged");
    }

    let output = cmd.output().context("Failed to execute git diff")?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("git diff failed: {}", err);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn execute(args: DiffArgs, client: &TypeSafeClient, model: &str) -> Result<()> {
    let diff_text = if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_string(&mut buffer)?;
        if buffer.trim().is_empty() {
            get_git_diff(args.staged)?
        } else {
            buffer
        }
    } else {
        get_git_diff(args.staged)?
    };

    if diff_text.trim().is_empty() {
        bail!("No git diff found to evaluate. Working directory appears clean.");
    }

    let state = serde_json::json!({
        "requested_prompt": args.prompt,
        "git_diff": diff_text
    });

    let mut questions = HashMap::new();

    questions.insert(
        "fulfills_prompt".to_string(),
        Question::Noul(NoulQuestion {
            instructions: serde_json::Value::String(
                "Does this git diff accurately and completely satisfy the requested prompt/requirements?"
                    .to_string(),
            ),
            criteria: Some(NoulCriteria {
                true_desc: Some("The diff directly implements what was requested without major omissions".to_string()),
                false_desc: Some("The diff misses core requirements or implements the wrong logic".to_string()),
            }),
        }),
    );

    questions.insert(
        "is_scope_clean".to_string(),
        Question::Noul(NoulQuestion {
            instructions: serde_json::Value::String(
                "Is this diff clean and disciplined, free from unrelated changes, stray edits, or accidental files?"
                    .to_string(),
            ),
            criteria: Some(NoulCriteria {
                true_desc: Some("Focused precisely on the task with no scope creep".to_string()),
                false_desc: Some("Contains unrelated edits, accidental files, or messy refactorings".to_string()),
            }),
        }),
    );

    questions.insert(
        "regression_risk".to_string(),
        Question::Score(ScoreQuestion {
            instructions: serde_json::Value::String(
                "What is the regression risk of these changes breaking existing behaviors?".to_string(),
            ),
            criteria: vec![
                "Negligible risk: changes are safe, isolated, or additive".to_string(),
                "Moderate risk: touches shared code or state, warrants careful review".to_string(),
                "High risk: dangerous mutations, breaking API changes, or potential crashes".to_string(),
            ],
        }),
    );

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
        let fulfills = match res.answers.get("fulfills_prompt") {
            Some(Answer::Noul(n)) => n.noul,
            _ => 0.0,
        };
        let clean = match res.answers.get("is_scope_clean") {
            Some(Answer::Noul(n)) => n.noul,
            _ => 0.0,
        };
        let risk = match res.answers.get("regression_risk") {
            Some(Answer::Score(s)) => s.score,
            _ => 2.0,
        };
        println!("fulfills={:.2} clean={:.2} risk={:.2}", fulfills, clean, risk);
        if fulfills < 0.65 || clean < 0.50 || risk >= 1.5 {
            std::process::exit(1);
        }
        return Ok(());
    }

    print_response_summary(&res, elapsed);

    let fulfills = match res.answers.get("fulfills_prompt") {
        Some(Answer::Noul(n)) => n.noul,
        _ => 0.0,
    };
    let clean = match res.answers.get("is_scope_clean") {
        Some(Answer::Noul(n)) => n.noul,
        _ => 0.0,
    };
    let risk = match res.answers.get("regression_risk") {
        Some(Answer::Score(s)) => s.score,
        _ => 2.0,
    };

    let passed = fulfills >= 0.65 && clean >= 0.50 && risk < 1.5;
    println!();
    if passed {
        println!("{}", "✔ Verification Gate Passed: Changes are ready to ship.".bright_green().bold());
    } else {
        println!("{}", "✘ Verification Gate Failed: Address issues before shipping.".bright_red().bold());
        std::process::exit(1);
    }

    Ok(())
}
