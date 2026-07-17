mod cli;
mod client;
mod collection;
mod config;
mod output;
mod resolver;

use std::io::{IsTerminal, Read};
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    match &cli.command {
        cli::Command::Auth { action } => return handle_auth(action),
        cli::Command::Run { path } => return handle_run(path).await,
        _ => {}
    }

    let cfg = config::Config::load();
    let http = client::HttpClient::new()?;

    let context = if !std::io::stdin().is_terminal() {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input).ok();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            None
        } else {
            serde_json::from_str(trimmed)
                .ok()
                .or_else(|| Some(serde_json::Value::String(trimmed.to_string())))
        }
    } else {
        None
    };

    let raw_url = cli.url();
    let url = resolver::resolve_template(
        &resolver::resolve_env(raw_url),
        context.as_ref(),
    );
    let body = cli.body().map(|b| {
        resolver::resolve_template(&resolver::resolve_env(b), context.as_ref())
    });

    let final_url = if !url.contains("://") && !url.starts_with("//") {
        if let Some(base) = &cfg.default_base {
            format!("{}/{}", base.trim_end_matches('/'), url.trim_start_matches('/'))
        } else {
            url
        }
    } else {
        url
    };

    let mut headers: Vec<(String, String)> = Vec::new();

    if let Some(ref name) = cli.auth {
        if let Some(profile) = cfg.find_auth(name) {
            let header_value = match profile.auth_type.as_str() {
                "bearer" => format!("Bearer {}", profile.value),
                "basic" => {
                    let encoded = base64_encode(&profile.value);
                    format!("Basic {encoded}")
                }
                _ => profile.value.clone(),
            };
            headers.push(("Authorization".to_string(), header_value));
        }
    }

    if cli.dry_run {
        println!("{} {}", cli.method()?.as_str().to_uppercase(), final_url);
        for (name, value) in &headers {
            println!("{name}: {value}");
        }
        if let Some(b) = &body {
            println!("\n--- body ---\n{b}");
        }
        return Ok(());
    }

    let method = cli.method()?;
    let response = http.request(method, &final_url, body.as_deref(), &headers).await?;
    let success = response.status().is_success();

    if let Some(extract_path) = cli.extract() {
        let body_text = response.text().await?;
        match serde_json::from_str::<serde_json::Value>(&body_text) {
            Ok(json) => match resolver::extract(&json, extract_path) {
                Some(val) => println!("{val}"),
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
        output::print_response(response, cli.output_mode()).await?;
    }

    if !success {
        std::process::exit(1);
    }
    Ok(())
}

async fn handle_run(path: &str) -> anyhow::Result<()> {
    let cfg = config::Config::load();
    let http = client::HttpClient::new()?;
    let results = collection::run_collection(path, &http, &cfg).await?;

    let mut all_ok = true;
    for r in &results {
        let icon = if r.success { "OK" } else { "FAIL" };

        let value_str = r.value.as_ref().map(|v| format!(" → {v}")).unwrap_or_default();
        println!("[{icon}] {status} {name}{value}",
            icon = icon,
            status = r.status,
            name = r.name,
            value = value_str,
        );
        if !r.success {
            all_ok = false;
        }
    }

    if !all_ok {
        std::process::exit(1);
    }
    Ok(())
}

fn handle_auth(action: &cli::AuthAction) -> anyhow::Result<()> {
    let mut cfg = config::Config::load();
    match action {
        cli::AuthAction::Add { name, auth_type, value } => {
            cfg.add_auth(name, auth_type, value);
            cfg.save()?;
            println!("Auth profile '{name}' saved");
        }
        cli::AuthAction::List => {
            if cfg.auth.is_empty() {
                println!("No auth profiles configured");
            } else {
                println!("Auth profiles:");
                for a in &cfg.auth {
                    println!("  {} (type: {}, value: {})", a.name, a.auth_type, "*".repeat(a.value.len().min(16)));
                }
            }
        }
        cli::AuthAction::Remove { name } => {
            if cfg.remove_auth(name) {
                cfg.save()?;
                println!("Auth profile '{name}' removed");
            } else {
                eprintln!("Auth profile '{name}' not found");
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

fn base64_encode(input: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input)
}
