use anyhow::Result;

use clap::{Parser, Subcommand};
use commands::backtest::*;

pub mod commands;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Backtest {
        #[command(flatten)]
        args: BacktestArgs,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backtest { args } => handle_backtest(args).await,
    }
}
