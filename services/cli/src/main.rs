use stakenet_cli::domain::CLI;

#[tokio::main]
async fn main() {
    let mut cli = CLI::new();
    cli.run();
}
