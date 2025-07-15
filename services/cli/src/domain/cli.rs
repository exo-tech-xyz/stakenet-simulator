use clap::{Parser, Subcommand, command};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Backtest { node_id: String },
}
pub struct CLI {}

impl CLI {
    pub fn new() -> Self {
        CLI {}
    }
    pub fn run(&mut self) {
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
                    Commands::Backtest { node_id } => self.backtest(node_id),
                },
                Err(e) => println!("That's not a valid command! Error: {}", e),
            };
        }
    }

    fn show_commands(&mut self) {
        println!(
            r#"COMMANDS:
    1) backtest <target_epoch> - creates a wallet and saves it into the wallets file. Returns the address.
    "#
        );
    }

    fn backtest(&mut self, target_epoch: String) {}
}
