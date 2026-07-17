use std::collections::HashMap;
use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Collection {
    pub client: Option<ClientConfig>,
    pub steps: Vec<StepConfig>,
}

#[derive(Deserialize)]
pub struct ClientConfig {
    pub base: Option<String>,
    pub auth: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Clone)]
pub struct StepConfig {
    pub name: Option<String>,
    #[serde(default = "default_method")]
    pub method: String,
    pub path: String,
    pub body: Option<serde_yaml::Value>,
    pub headers: Option<HashMap<String, String>>,
    pub status: Option<u16>,
    pub extract: Option<String>,
}

fn default_method() -> String {
    "GET".to_string()
}

pub struct StepResult {
    pub name: String,
    pub status: u16,
    pub value: Option<String>,
    pub success: bool,
}

fn serialize_body(body: &serde_yaml::Value) -> String {
    match body {
        serde_yaml::Value::String(s) => s.clone(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

pub async fn run_collection(
    path: &str,
    http: &crate::client::HttpClient,
    cfg: &crate::config::Config,
) -> anyhow::Result<Vec<StepResult>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {path}"))?;
    let collection: Collection = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse {path}"))?;

    let base = collection.client.as_ref()
        .and_then(|c| c.base.as_deref())
        .or(cfg.default_base.as_deref());
    let client_auth = collection.client.as_ref()
        .and_then(|c| c.auth.as_deref());
    let client_headers = collection.client.as_ref()
        .and_then(|c| c.headers.as_ref());

    let mut step_context: HashMap<String, serde_json::Value> = HashMap::new();
    let mut results = Vec::new();

    for step in &collection.steps {
        let step_name = step.name.clone().unwrap_or_else(|| step.method.clone());
        let ctx_val = if step_context.is_empty() {
            None
        } else {
            Some(step_context_to_value(&step_context))
        };

        let resolved_path = crate::resolver::resolve_template(
            &crate::resolver::resolve_env(&step.path),
            ctx_val.as_ref(),
        );

        let url = if !resolved_path.contains("://") {
            if let Some(base) = base {
                format!("{}/{}", base.trim_end_matches('/'), resolved_path.trim_start_matches('/'))
            } else {
                resolved_path
            }
        } else {
            resolved_path
        };

        let body_str = step.body.as_ref().map(|b| {
            let raw = serialize_body(b);
            crate::resolver::resolve_template(&crate::resolver::resolve_env(&raw), ctx_val.as_ref())
        });

        let mut headers: Vec<(String, String)> = Vec::new();

        if let Some(ch) = client_headers {
            for (k, v) in ch {
                headers.push((k.clone(), v.clone()));
            }
        }
        if let Some(ref h) = step.headers {
            for (k, v) in h {
                let val = crate::resolver::resolve_template(&crate::resolver::resolve_env(v), ctx_val.as_ref());
                headers.push((k.clone(), val));
            }
        }

        let auth_name = client_auth.or(step.name.as_deref());
        if let Some(auth_name) = auth_name {
            if let Some(profile) = cfg.find_auth(auth_name) {
                let auth_value = match profile.auth_type.as_str() {
                    "bearer" => format!("Bearer {}", profile.value),
                    "basic" => {
                        let encoded = crate::base64_encode(&profile.value);
                        format!("Basic {encoded}")
                    }
                    _ => profile.value.clone(),
                };
                headers.push(("Authorization".to_string(), auth_value));
            }
        }

        let method: reqwest::Method = step.method.to_uppercase().parse()
            .unwrap_or(reqwest::Method::GET);

        let response = http.request(method, &url, body_str.as_deref(), &headers).await?;
        let status = response.status().as_u16();
        let body_text = response.text().await?;

        let mut step_ok = true;

        if let Some(expected) = step.status {
            if status != expected {
                eprintln!("[FAIL] {step_name}: expected status {expected}, got {status}");
                step_ok = false;
            }
        }

        let value = step.extract.as_ref().and_then(|extract_path| {
            serde_json::from_str::<serde_json::Value>(&body_text).ok()
                .and_then(|json| crate::resolver::extract(&json, extract_path))
        });

        if let Some(ref val) = value {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(val) {
                step_context.insert(step_name.clone(), parsed);
            } else {
                step_context.insert(step_name.clone(), serde_json::Value::String(val.clone()));
            }
        }

        results.push(StepResult {
            name: step_name,
            status,
            value,
            success: step_ok,
        });
    }

    Ok(results)
}

fn step_context_to_value(ctx: &HashMap<String, serde_json::Value>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in ctx {
        map.insert(k.clone(), v.clone());
    }
    serde_json::Value::Object(map)
}
