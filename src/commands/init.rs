use anyhow::Result;
use colored::Colorize;
use crate::config::{write_config, Config};
use std::path::PathBuf;

pub async fn run() -> Result<()> {
    let path = PathBuf::from("yei.json");
    if path.exists() {
        println!("{} yei.json already exists", "⚠".yellow());
        return Ok(());
    }
    let config = Config::default();
    write_config(&config)?;
    println!("{} Created yei.json", "✓".green());
    println!("  Edit {} to set your registry URL and output path.", "yei.json".cyan());
    Ok(())
}
