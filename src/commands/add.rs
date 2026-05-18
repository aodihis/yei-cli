use anyhow::{bail, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use crate::config::{read_config, read_lock, write_lock};
use crate::registry::Client;

pub async fn run(components: Vec<String>, version_override: Option<String>) -> Result<()> {
    if components.is_empty() {
        bail!("Specify at least one component name. Run {} to see available components.", "yei list".cyan());
    }

    let config = read_config()?;
    let mut lock = read_lock()?;
    let client = Client::new(&config.registry);
    let out = PathBuf::from(&config.output_path);

    fs::create_dir_all(&out)?;

    for raw in &components {
        let (name, version) = parse_component_arg(raw, &version_override, &config.version);

        println!("{} Fetching {}{}...", "→".cyan(), name,
            if version == "latest" { String::new() } else { format!("@{version}") });

        let source = client.fetch_component(&name, &version).await?;

        let dest = out.join(format!("{name}.rs"));
        fs::write(&dest, &source)?;

        lock.components.insert(name.clone(), version.clone());

        println!("{} Added {} → {}", "✓".green(), name.green().bold(), dest.display());
        println!("  Add to your mod.rs:  {}", format!("pub mod {name};").cyan());

        // Print any cargo deps from registry metadata
        let reg = client.fetch_registry(&version).await;
        if let Ok(registry) = reg {
            if let Some(meta) = registry.components.iter().find(|c| c.name == name) {
                if !meta.cargo_deps.is_empty() {
                    println!("  Add to Cargo.toml:");
                    for dep in &meta.cargo_deps {
                        if dep.features.is_empty() {
                            println!("    {} = \"{}\"", dep.name, dep.version);
                        } else {
                            println!("    {} = {{ version = \"{}\", features = {:?} }}",
                                dep.name, dep.version, dep.features);
                        }
                    }
                }
            }
        }
    }

    write_lock(&lock)?;
    Ok(())
}

fn parse_component_arg(raw: &str, version_override: &Option<String>, default_version: &str) -> (String, String) {
    if let Some(at) = raw.find('@') {
        let name = raw[..at].to_string();
        let version = raw[at + 1..].to_string();
        (name, version)
    } else {
        let version = version_override.clone().unwrap_or_else(|| default_version.to_string());
        (raw.to_string(), version)
    }
}
