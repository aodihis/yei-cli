use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Context, Result};

const CONFIG_FILE: &str = "yei.json";
const LOCK_FILE: &str = "yei.lock";
const DEFAULT_REGISTRY: &str = "https://yei.yourdomain.com";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub registry: String,
    pub version: String,
    pub output_path: String,
    /// Rust module path where components live (e.g. "crate::components").
    /// Derived from output_path if not set explicitly.
    #[serde(default)]
    pub module_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            registry: DEFAULT_REGISTRY.to_string(),
            version: "latest".to_string(),
            output_path: "src/components".to_string(),
            module_path: String::new(),
        }
    }
}

impl Config {
    /// Returns the effective module path, deriving it from output_path if not
    /// set explicitly. "src/foo/bar" → "crate::foo::bar".
    pub fn effective_module_path(&self) -> String {
        if !self.module_path.is_empty() {
            return self.module_path.clone();
        }
        let stripped = self.output_path
            .trim_start_matches("src/")
            .trim_start_matches("src\\");
        format!("crate::{}", stripped.replace('\\', "::").replace('/', "::"))
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LockFile {
    pub components: HashMap<String, String>,
}

pub fn read_config() -> Result<Config> {
    let path = PathBuf::from(CONFIG_FILE);
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {CONFIG_FILE}"))?;
    serde_json::from_str(&content).with_context(|| format!("Failed to parse {CONFIG_FILE}"))
}

pub fn write_config(config: &Config) -> Result<()> {
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(CONFIG_FILE, content)?;
    Ok(())
}

pub fn read_lock() -> Result<LockFile> {
    let path = PathBuf::from(LOCK_FILE);
    if !path.exists() {
        return Ok(LockFile::default());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {LOCK_FILE}"))?;
    serde_json::from_str(&content).with_context(|| format!("Failed to parse {LOCK_FILE}"))
}

pub fn write_lock(lock: &LockFile) -> Result<()> {
    let content = serde_json::to_string_pretty(lock)?;
    std::fs::write(LOCK_FILE, content)?;
    Ok(())
}
