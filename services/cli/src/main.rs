use std::sync::Arc;

use anyhow::Result;

use clap::{Parser, Subcommand};
use commands::backtest::*;
use sqlx::postgres::PgPoolOptions;

use crate::domain::Config;

pub mod commands;
pub mod domain;

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

    let config = Config::from_env()?;

    let db_conn_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.db_connection_url)
            .await
            .unwrap(),
    );

    match cli.command {
        Commands::Backtest { args } => handle_backtest(args, &db_conn_pool).await,
    }
}
