use anyhow::Result;
use colored::Colorize;
use crate::config::read_config;
use crate::registry::Client;

pub async fn run() -> Result<()> {
    let config = read_config()?;
    let client = Client::new(&config.registry);
    let registry = client.fetch_registry(&config.version).await?;

    println!("{} {} components available (registry v{})\n",
        "yei".cyan().bold(), registry.components.len(), registry.version);

    for comp in &registry.components {
        println!("  {} — {}", comp.name.green().bold(), comp.description);
    }
    println!("\nRun {} to add a component.", "yei add <name>".cyan());
    Ok(())
}
