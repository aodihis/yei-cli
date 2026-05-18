use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CargoDep {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComponentMeta {
    pub name: String,
    pub description: String,
    pub files: Vec<String>,
    pub cargo_deps: Vec<CargoDep>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Registry {
    pub version: String,
    pub components: Vec<ComponentMeta>,
}

pub struct Client {
    base_url: String,
    http: reqwest::Client,
}

impl Client {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn fetch_registry(&self, version: &str) -> Result<Registry> {
        let url = if version == "latest" {
            format!("{}/registry", self.base_url)
        } else {
            format!("{}/registry/{}", self.base_url, version)
        };
        let resp = self.http.get(&url).send().await
            .with_context(|| format!("Failed to reach registry at {url}"))?;
        if !resp.status().is_success() {
            bail!("Registry returned {}: {url}", resp.status());
        }
        resp.json::<Registry>().await.context("Failed to parse registry response")
    }

    pub async fn fetch_component(&self, name: &str, version: &str) -> Result<String> {
        let url = if version == "latest" {
            format!("{}/component/{}", self.base_url, name)
        } else {
            format!("{}/component/{}/{}", self.base_url, name, version)
        };
        let resp = self.http.get(&url).send().await
            .with_context(|| format!("Failed to fetch component {name}"))?;
        if !resp.status().is_success() {
            bail!("Component '{name}' not found at version {version}");
        }
        resp.text().await.context("Failed to read component source")
    }

    pub async fn fetch_versions(&self) -> Result<Vec<String>> {
        let url = format!("{}/versions", self.base_url);
        let resp = self.http.get(&url).send().await
            .with_context(|| format!("Failed to reach {url}"))?;
        if !resp.status().is_success() {
            bail!("Server returned {}", resp.status());
        }
        resp.json::<Vec<String>>().await.context("Failed to parse versions")
    }
}
