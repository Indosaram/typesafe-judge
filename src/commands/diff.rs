use anyhow::{bail, Context, Result};
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::io::{self, IsTerminal, Read};
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

    #[arg(long)]
    pub untracked: bool,

    #[arg(short, long)]
    pub revision: Option<String>,

    #[arg(long)]
    pub path: Option<String>,

    #[arg(long, default_value_t = 0.65)]
    pub fulfills_threshold: f64,

    #[arg(long, default_value_t = 0.50)]
    pub clean_threshold: f64,

    #[arg(long, default_value_t = 1.50)]
    pub max_risk: f64,

    #[arg(short, long)]
    pub quiet: bool,

    #[arg(long)]
    pub json: bool,
}

fn get_git_diff(staged: bool, revision: Option<&str>, path_filter: Option<&str>, include_untracked: bool) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.arg("diff");

    if staged {
        cmd.arg("--staged");
    }

    if let Some(rev) = revision {
        cmd.arg(rev);
    }

    if let Some(p) = path_filter {
        cmd.arg("--").arg(p);
    }

    let output = cmd.output().context("Failed to execute git diff")?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("git diff failed: {}", err);
    }

    let mut diff_text = String::from_utf8_lossy(&output.stdout).to_string();

    if include_untracked {
        let mut untracked_cmd = Command::new("git");
        untracked_cmd.args(["ls-files", "--others", "--exclude-standard"]);
        if let Ok(untracked_out) = untracked_cmd.output() {
            let files_str = String::from_utf8_lossy(&untracked_out.stdout);
            let untracked_files: Vec<&str> = files_str.lines().filter(|l| !l.trim().is_empty()).collect();
            if !untracked_files.is_empty() {
                diff_text.push_str("\n\n--- Untracked Files ---\n");
                for f in untracked_files {
                    diff_text.push_str(&format!("Untracked: {}\n", f));
                }
            }
        }
    }

    Ok(diff_text)
}

pub async fn execute(args: DiffArgs, client: &TypeSafeClient, model: &str) -> Result<()> {
    let diff_text = if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_string(&mut buffer)?;
        if buffer.trim().is_empty() {
            get_git_diff(args.staged, args.revision.as_deref(), args.path.as_deref(), args.untracked)?
        } else {
            buffer
        }
    } else {
        get_git_diff(args.staged, args.revision.as_deref(), args.path.as_deref(), args.untracked)?
    };

    if diff_text.trim().is_empty() {
        bail!("No git diff found to evaluate. Working tree appears clean.");
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

    if args.quiet {
        println!("fulfills={:.2} clean={:.2} risk={:.2}", fulfills, clean, risk);
        if fulfills < args.fulfills_threshold || clean < args.clean_threshold || risk >= args.max_risk {
            std::process::exit(1);
        }
        return Ok(());
    }

    print_response_summary(&res, elapsed);

    let passed = fulfills >= args.fulfills_threshold && clean >= args.clean_threshold && risk < args.max_risk;
    println!();
    if passed {
        println!("{}", "✔ Verification Gate Passed: Changes are ready to ship.".bright_green().bold());
    } else {
        println!("{}", "✘ Verification Gate Failed: Address issues before shipping.".bright_red().bold());
        std::process::exit(1);
    }

    Ok(())
}
