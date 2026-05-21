use anyhow::{bail, Result};
use colored::Colorize;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use crate::config::{read_config, read_lock, write_lock, Config, LockFile};
use crate::registry::{Client, Registry};

/// The module path used in component source files on the registry.
const REGISTRY_MODULE_PATH: &str = "crate::components";

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

    // Resolve full install order: deps first, then requested components (BFS).
    let mut queue: VecDeque<String> = components.iter().map(|r| parse_name(r)).collect();
    let mut ordered: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    while let Some(name) = queue.pop_front() {
        if seen.contains(&name) { continue; }
        seen.insert(name.clone());

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
        install_one(name, &version, &client, &registry, &out, &mut lock, &config).await?;
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
    config: &Config,
) -> Result<()> {
    if lock.components.contains_key(name) {
        println!("{} {} already installed", "·".dimmed(), name);
        return Ok(());
    }

    println!("{} Fetching {}{}...", "→".cyan(), name,
        if version == "latest" { String::new() } else { format!("@{version}") });

    let source = client.fetch_component(name, version).await?;
    let source = rewrite_imports(source, &config.effective_module_path());
    let dest = out.join(format!("{name}.rs"));
    fs::write(&dest, &source)?;

    lock.components.insert(name.to_string(), version.to_string());

    // Append to mod.rs if it exists and the entry isn't already there
    let mod_path = out.join("mod.rs");
    let mod_line = format!("pub mod {name};\n");
    if mod_path.exists() {
        let existing = fs::read_to_string(&mod_path)?;
        if !existing.contains(&mod_line.trim_end().to_string()) {
            let mut file = fs::OpenOptions::new().append(true).open(&mod_path)?;
            use std::io::Write;
            file.write_all(mod_line.as_bytes())?;
        }
    }

    println!("{} Added {} → {}", "✓".green(), name.green().bold(), dest.display());

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

/// Replaces the registry's hardcoded module path with the user's configured one.
fn rewrite_imports(source: String, module_path: &str) -> String {
    if module_path == REGISTRY_MODULE_PATH {
        return source;
    }
    source.replace(
        &format!("use {REGISTRY_MODULE_PATH}::"),
        &format!("use {module_path}::"),
    )
}

fn parse_name(raw: &str) -> String {
    match raw.find('@') {
        Some(at) => raw[..at].to_string(),
        None => raw.to_string(),
    }
}
