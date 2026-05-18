use anyhow::Result;
use colored::Colorize;
use crate::config::read_config;
use crate::registry::Client;

pub async fn run() -> Result<()> {
    let config = read_config()?;
    let client = Client::new(&config.registry);
    let versions = client.fetch_versions().await?;

    println!("{} Available versions:\n", "yei".cyan().bold());
    for v in &versions {
        if v == "latest" {
            println!("  {} {}", v.green().bold(), "(alias)".dimmed());
        } else {
            println!("  {}", v.green());
        }
    }
    Ok(())
}
