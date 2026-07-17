mod cli;
mod client;
mod output;
mod resolver;

use std::io::{IsTerminal, Read};
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let client = client::HttpClient::new()?;

    let context = if !std::io::stdin().is_terminal() {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input).ok();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            None
        } else {
            serde_json::from_str(trimmed).ok()
                .or_else(|| Some(serde_json::Value::String(trimmed.to_string())))
        }
    } else {
        None
    };

    let url = resolver::resolve_template(cli.url(), context.as_ref());
    let body = cli.body()
        .map(|b| resolver::resolve_template(b, context.as_ref()));

    let method = cli.method()?;
    let response = client.request(method, &url, body.as_deref()).await?;
    let success = response.status().is_success();

    if let Some(extract_path) = cli.extract() {
        let body_text = response.text().await?;
        match serde_json::from_str::<serde_json::Value>(&body_text) {
            Ok(json) => match resolver::extract(&json, extract_path) {
                Some(val) => {
                    println!("{val}");
                }
                None => {
                    eprintln!("Extract path '{extract_path}' not found");
                    std::process::exit(1);
                }
            },
            Err(_) => {
                eprintln!("Response is not JSON, cannot extract");
                std::process::exit(1);
            }
        }
    } else {
        let mode = cli.output_mode();
        output::print_response(response, mode).await?;
    }

    if !success {
        std::process::exit(1);
    }
    Ok(())
}
