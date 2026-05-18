use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use crate::config::{read_config, read_lock, write_lock};
use crate::registry::Client;

pub async fn run() -> Result<()> {
    let config = read_config()?;
    let mut lock = read_lock()?;

    if lock.components.is_empty() {
        println!("{} No components installed yet. Run {} first.", "ℹ".blue(), "yei add <name>".cyan());
        return Ok(());
    }

    let client = Client::new(&config.registry);
    let out = PathBuf::from(&config.output_path);

    println!("{} Upgrading {} component(s) to latest...\n", "→".cyan(), lock.components.len());

    let names: Vec<String> = lock.components.keys().cloned().collect();
    for name in names {
        let source = client.fetch_component(&name, "latest").await?;
        let dest = out.join(format!("{name}.rs"));
        fs::write(&dest, &source)?;
        lock.components.insert(name.clone(), "latest".to_string());
        println!("  {} {}", "✓".green(), name);
    }

    write_lock(&lock)?;
    println!("\n{} All components upgraded.", "✓".green().bold());
    Ok(())
}
