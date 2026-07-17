mod cli;
mod client;
mod output;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let client = client::HttpClient::new()?;
    let mode = cli.output_mode();
    let response = client.request(cli.method()?, cli.url(), cli.body()).await?;
    let success = response.status().is_success();
    output::print_response(response, mode).await?;
    if !success {
        std::process::exit(1);
    }
    Ok(())
}
