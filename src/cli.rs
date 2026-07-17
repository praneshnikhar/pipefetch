use clap::Parser;

#[derive(Parser)]
#[command(name = "pipefetch", about = "HTTP client for shell pipelines")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Print only the status line
    #[arg(long, global = true)]
    pub status: bool,

    /// Print only response headers
    #[arg(long, global = true)]
    pub headers: bool,

    /// Print raw response body (no formatting)
    #[arg(long, global = true)]
    pub raw: bool,

    /// Machine-readable JSON output
    #[arg(long, global = true)]
    pub json: bool,

    /// Extract a value from the JSON response using dot notation (e.g. .id, .users[0].name)
    #[arg(long, global = true)]
    pub extract: Option<String>,

    /// Name of an auth profile to use (stored in ~/.config/pipefetch/config.yaml)
    #[arg(long, global = true)]
    pub auth: Option<String>,

    /// Print what would be sent without making the request
    #[arg(long, global = true)]
    pub dry_run: bool,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Send a GET request
    Get { url: String },
    /// Send a POST request
    Post { url: String, body: String },
    /// Send a PUT request
    Put { url: String, body: String },
    /// Send a PATCH request
    Patch { url: String, body: String },
    /// Send a DELETE request
    Delete { url: String },
    /// Manage auth profiles
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
}

#[derive(clap::Subcommand)]
pub enum AuthAction {
    /// Add or update an auth profile
    Add {
        name: String,
        #[arg(long)]
        auth_type: String,
        #[arg(long)]
        value: String,
    },
    /// List all auth profiles
    List,
    /// Remove an auth profile
    Remove { name: String },
}

impl Cli {
    pub fn method(&self) -> anyhow::Result<reqwest::Method> {
        Ok(match &self.command {
            Command::Get { .. } => reqwest::Method::GET,
            Command::Post { .. } => reqwest::Method::POST,
            Command::Put { .. } => reqwest::Method::PUT,
            Command::Patch { .. } => reqwest::Method::PATCH,
            Command::Delete { .. } => reqwest::Method::DELETE,
            Command::Auth { .. } => unreachable!(),
        })
    }

    pub fn url(&self) -> &str {
        match &self.command {
            Command::Get { url } => url,
            Command::Post { url, .. } => url,
            Command::Put { url, .. } => url,
            Command::Patch { url, .. } => url,
            Command::Delete { url } => url,
            Command::Auth { .. } => "",
        }
    }

    pub fn body(&self) -> Option<&str> {
        match &self.command {
            Command::Post { body, .. } => Some(body.as_str()),
            Command::Put { body, .. } => Some(body.as_str()),
            Command::Patch { body, .. } => Some(body.as_str()),
            _ => None,
        }
    }

    pub fn extract(&self) -> Option<&str> {
        self.extract.as_deref()
    }

    pub fn output_mode(&self) -> crate::output::OutputMode {
        if self.status {
            crate::output::OutputMode::Status
        } else if self.headers {
            crate::output::OutputMode::Headers
        } else if self.raw {
            crate::output::OutputMode::Raw
        } else if self.json {
            crate::output::OutputMode::Json
        } else {
            crate::output::OutputMode::Default
        }
    }
}
