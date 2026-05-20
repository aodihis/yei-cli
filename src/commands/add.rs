use anyhow::{bail, Result};
use colored::Colorize;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use crate::config::{read_config, read_lock, write_lock, LockFile};
use crate::registry::{Client, Registry};

pub async fn run(components: Vec<String>, version_override: Option<String>) -> Result<()> {
    if components.is_empty() {
        bail!("Specify at least one component name. Run {} to see available components.", "yei list".cyan());
    }

    let config = read_config()?;
    let mut lock = read_lock()?;
    let client = Client::new(&config.registry);
    let out = PathBuf::from(&config.output_path);

    fs::create_dir_all(&out)?;

    let version = version_override.unwrap_or_else(|| config.version.clone());
    let registry = client.fetch_registry(&version).await?;

    // Resolve the full install order: deps first, then the requested components.
    // Uses BFS so each name appears only once and deps always precede dependents.
    let mut queue: VecDeque<String> = components
        .iter()
        .map(|raw| parse_name(raw))
        .collect();
    let mut ordered: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    while let Some(name) = queue.pop_front() {
        if seen.contains(&name) { continue; }
        seen.insert(name.clone());

        // Push deps before the component itself
        if let Some(meta) = registry.components.iter().find(|c| c.name == name) {
            for dep in &meta.deps {
                if !seen.contains(dep) {
                    queue.push_front(dep.clone());
                }
            }
        }

        ordered.push(name);
    }

    for name in &ordered {
        install_one(name, &version, &client, &registry, &out, &mut lock).await?;
    }

    write_lock(&lock)?;
    Ok(())
}

async fn install_one(
    name: &str,
    version: &str,
    client: &Client,
    registry: &Registry,
    out: &PathBuf,
    lock: &mut LockFile,
) -> Result<()> {
    if lock.components.contains_key(name) {
        println!("{} {} already installed", "·".dimmed(), name);
        return Ok(());
    }

    println!("{} Fetching {}{}...", "→".cyan(), name,
        if version == "latest" { String::new() } else { format!("@{version}") });

    let source = client.fetch_component(name, version).await?;
    let dest = out.join(format!("{name}.rs"));
    fs::write(&dest, &source)?;

    lock.components.insert(name.to_string(), version.to_string());

    println!("{} Added {} → {}", "✓".green(), name.green().bold(), dest.display());
    println!("  Add to your mod.rs:  {}", format!("pub mod {name};").cyan());

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

    Ok(())
}

fn parse_name(raw: &str) -> String {
    match raw.find('@') {
        Some(at) => raw[..at].to_string(),
        None => raw.to_string(),
    }
}
