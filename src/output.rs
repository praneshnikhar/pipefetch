use anyhow::Context;
use colored::*;

pub enum OutputMode {
    Default,
    Status,
    Headers,
    Raw,
    Json,
}

pub async fn print_response(response: reqwest::Response, mode: OutputMode) -> anyhow::Result<()> {
    let status = response.status();
    let reason = status.canonical_reason().unwrap_or("Unknown");
    let status_line = format!("{} {}", status.as_u16(), reason);
    let headers = response.headers().clone();
    let body = response.text().await.context("Failed to read response body")?;

    match mode {
        OutputMode::Status => {
            println!("{}", status_line.color(status_color(status.as_u16())));
        }
        OutputMode::Headers => {
            println!("{}", status_line.bold());
            for (name, value) in &headers {
                let val = value.to_str().unwrap_or("<binary>");
                println!("{}: {}", name.as_str().cyan(), val);
            }
        }
        OutputMode::Raw => {
            print!("{}", body);
            if !body.ends_with('\n') {
                println!();
            }
        }
        OutputMode::Json => {
            let json_headers: serde_json::Value = headers
                .iter()
                .map(|(k, v)| {
                    (
                        k.as_str().to_string(),
                        serde_json::Value::String(v.to_str().unwrap_or("").to_string()),
                    )
                })
                .collect();
            let parsed_body = serde_json::from_str::<serde_json::Value>(&body)
                .unwrap_or(serde_json::Value::String(body));
            let json = serde_json::json!({
                "status": status.as_u16(),
                "headers": json_headers,
                "body": parsed_body,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputMode::Default => {
            println!("{}", status_line.bold());
            for (name, value) in &headers {
                let val = value.to_str().unwrap_or("<binary>");
                println!("{}: {}", name.as_str().cyan(), val);
            }
            println!();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                print!("{}", body);
                if !body.ends_with('\n') {
                    println!();
                }
            }
        }
    }
    Ok(())
}

fn status_color(code: u16) -> Color {
    match code {
        200..=299 => Color::Green,
        300..=399 => Color::Yellow,
        400..=499 => Color::Red,
        500..=599 => Color::BrightRed,
        _ => Color::White,
    }
}
