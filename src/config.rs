use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub default_base: Option<String>,
    pub default_timeout: Option<u64>,
    #[serde(default)]
    pub auth: Vec<AuthProfile>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthProfile {
    pub name: String,
    #[serde(rename = "type")]
    pub auth_type: String,
    pub value: String,
}

impl Config {
    pub fn path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        base.join("pipefetch").join("config.yaml")
    }

    pub fn load() -> Self {
        let p = Self::path();
        if p.exists() {
            std::fs::read_to_string(&p)
                .ok()
                .and_then(|c| serde_yaml::from_str(&c).ok())
                .unwrap_or_default()
        } else {
            Config::default()
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let p = Self::path();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&p, serde_yaml::to_string(self)?)?;
        Ok(())
    }

    pub fn find_auth(&self, name: &str) -> Option<&AuthProfile> {
        self.auth.iter().find(|a| a.name == name)
    }

    pub fn add_auth(&mut self, name: &str, auth_type: &str, value: &str) {
        self.auth.retain(|a| a.name != name);
        self.auth.push(AuthProfile {
            name: name.to_string(),
            auth_type: auth_type.to_string(),
            value: value.to_string(),
        });
    }

    pub fn remove_auth(&mut self, name: &str) -> bool {
        let len = self.auth.len();
        self.auth.retain(|a| a.name != name);
        self.auth.len() < len
    }
}
