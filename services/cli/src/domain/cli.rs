use clap::{Parser, Subcommand, command};
use sqlx::{Error as SqlxError, postgres::PgPoolOptions};
use std::{str::FromStr, sync::Arc};
use tracing::{Level, error};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Backtest {
        mev_commission_score: String,
        commission_score: String,
        historical_commission_score: String,
        blacklisted_score: String,
        superminority_score: String,
        delinquency_score: String,
        running_jito_score: String,
        yield_score: String,
        merkle_root_upload_authority_score: String,
        priority_fee_commission_score: String,
        target_epoch: String,
    },
}
pub struct CLI {}

impl CLI {
    pub fn new() -> Self {
        CLI {}
    }
    #[tokio::main]
    pub async fn run(&mut self) {
        let level = std::env::var("RUST_LOG").unwrap_or(Level::INFO.to_string());
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(EnvFilter::new(level))
            // this needs to be set to remove duplicated information in the log.
            .with_current_span(false)
            // this needs to be set to false, otherwise ANSI color codes will
            // show up in a confusing manner in CloudWatch logs.
            .with_ansi(false)
            // disabling time is handy because CloudWatch will add the ingestion time.
            .without_time()
            // remove the name of the function from every log entry
            .with_target(false)
            .init();

        let config = Config::from_env()?;

        let db_conn_pool = Arc::new(
            PgPoolOptions::new()
                .max_connections(5)
                .connect(&config.db_connection_url)
                .await
                .unwrap(),
        );
        loop {
            self.show_commands();
            let mut buf = String::new();
            std::io::stdin()
                .read_line(&mut buf)
                .expect("Couldn't parse stdin");
            let line = buf.trim();
            let mut args = vec!["program".to_string()];
            args.extend(shlex::split(line).ok_or("error: Invalid quoting").unwrap());
            match Args::try_parse_from(args) {
                Ok(cli) => match cli.cmd {
                    Commands::Backtest {
                        mev_commission_score,
                        commission_score,
                        historical_commission_score,
                        blacklisted_score,
                        superminority_score,
                        delinquency_score,
                        running_jito_score,
                        yield_score,
                        merkle_root_upload_authority_score,
                        priority_fee_commission_score,
                        target_epoch,
                    } => self.backtest(target_epoch),
                },
                Err(e) => println!("That's not a valid command! Error: {}", e),
            };
        }
    }

    fn show_commands(&mut self) {
        println!(
            r#"COMMANDS:
    1) backtest <target_epoch> - It takes in parameter overrides (current parameters could be the defaults) and the target epoch (that the scoring/evaluation criteria is ran from). It should then use the data in the PostgreSQL DB to run the Steward’s scoring criteria ranking all validators and determine which validators made it into the X (also a simulation parameter) number delegated to by jitoSOL. Then use that list to determine the APY for SOL staked across those validators over a given epoch period. In this calculation assume the SOL is 100% activated across all epochs. 
    "#
        );
    }

    fn backtest(&mut self, target_epoch: String) {
        println!("Fetching data from DB");
        println!("Running Steward’s scoring criteria");
        println!(
            "determine the APY for SOL staked across those validators over a given epoch period"
        );
    }
}
