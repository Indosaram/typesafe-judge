mod client;
mod commands;
mod formatter;
mod input;
mod models;

use anyhow::Result;
use clap::{Parser, Subcommand};

use client::TypeSafeClient;
use commands::{choice, diff, eval, noul, score};

#[derive(Parser, Debug)]
#[command(
    name = "typesafe-judge",
    about = "Fast decision engine for coding tasks, verification gates, and agent workflows using TypeSafe System One (Jev)",
    version
)]
struct Cli {
    #[arg(long, global = true, env = "TYPESAFE_API_KEY", hide_env_values = true)]
    api_key: Option<String>,

    #[arg(long, global = true, default_value = "jev-latest")]
    model: String,

    #[arg(long, global = true)]
    url: Option<String>,

    #[arg(long, global = true)]
    pretty: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Choice(choice::ChoiceArgs),
    Noul(noul::NoulArgs),
    Score(score::ScoreArgs),
    Diff(diff::DiffArgs),
    Eval(eval::EvalArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let client = TypeSafeClient::new(cli.api_key, cli.url)?;

    match cli.command {
        Commands::Choice(args) => choice::execute(args, &client, &cli.model, cli.pretty).await,
        Commands::Noul(args) => noul::execute(args, &client, &cli.model, cli.pretty).await,
        Commands::Score(args) => score::execute(args, &client, &cli.model, cli.pretty).await,
        Commands::Diff(args) => diff::execute(args, &client, &cli.model, cli.pretty).await,
        Commands::Eval(args) => eval::execute(args, &client, &cli.model, cli.pretty).await,
    }
}
