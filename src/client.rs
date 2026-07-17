use anyhow::Context;

pub struct HttpClient {
    inner: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> anyhow::Result<Self> {
        let inner = reqwest::Client::builder()
            .user_agent(concat!("pipefetch/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Failed to create HTTP client")?;
        Ok(Self { inner })
    }

    pub async fn request(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<&str>,
    ) -> anyhow::Result<reqwest::Response> {
        let mut req = self.inner.request(method, url);
        if let Some(body) = body {
            req = req
                .header("Content-Type", "application/json")
                .body(body.to_owned());
        }
        let response = req
            .send()
            .await
            .with_context(|| format!("Request to {url} failed"))?;
        Ok(response)
    }
}
