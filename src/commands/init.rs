use anyhow::{Context, Result};

const YEI_ROOT_TOKENS: &str = r#":root {
  --background:            oklch(1 0 0);
  --foreground:            oklch(0.145 0 0);

  --primary:               oklch(0.205 0 0);
  --primary-foreground:    oklch(0.985 0 0);

  --secondary:             oklch(0.97 0 0);
  --secondary-foreground:  oklch(0.205 0 0);

  --danger:                oklch(0.577 0.245 27.325);
  --on-danger:             oklch(0.985 0 0);

  --muted:                 oklch(0.97 0 0);
  --muted-foreground:      oklch(0.205 0 0);

  --success:               oklch(0.527 0.154 150);
  --warning:               oklch(0.769 0.188 70);

  --field:                 oklch(0.97 0 0);
  --border:                oklch(0.922 0 0);
  --focus:                 oklch(0.708 0 0);

  --radius:                0.625rem;
}
"#;
use colored::Colorize;
use dialoguer::{Input, theme::ColorfulTheme};
use std::path::{Path, PathBuf};
use crate::config::{write_config, Config};
use crate::registry::Client;

pub async fn run() -> Result<()> {
    let config_path = PathBuf::from("yei.json");
    if config_path.exists() {
        println!("{} yei.json already exists — delete it to re-run init", "⚠".yellow());
        return Ok(());
    }

    println!("{}", "Setting up yei...".bold());
    println!();

    let theme = ColorfulTheme::default();

    let output_path: String = Input::with_theme(&theme)
        .with_prompt("Component output directory")
        .default("src/components".to_string())
        .interact_text()?;

    let css_entry: String = Input::with_theme(&theme)
        .with_prompt("CSS entry file")
        .default("src/style.css".to_string())
        .interact_text()?;

    println!();

    // --- Download yei.css ---
    let config = Config {
        registry: Config::default().registry,
        version: "latest".to_string(),
        output_path: output_path.clone(),
    };
    let client = Client::new(&config.registry);
    let css_content = client.fetch_style().await
        .context("Could not download yei.css — check your registry URL in yei.json after init")?;

    // Place yei.css next to the CSS entry file
    let css_entry_path = Path::new(&css_entry);
    let css_dir = css_entry_path.parent().unwrap_or(Path::new("."));
    let yei_css_path = css_dir.join("yei.css");
    std::fs::create_dir_all(css_dir)
        .with_context(|| format!("Could not create directory {}", css_dir.display()))?;
    std::fs::write(&yei_css_path, &css_content)
        .with_context(|| format!("Could not write {}", yei_css_path.display()))?;
    println!("{} Downloaded yei.css → {}", "✓".green(), yei_css_path.display());

    // --- Update or create the CSS entry file ---
    let import_line = r#"@import "./yei.css";"#;
    if css_entry_path.exists() {
        let existing = std::fs::read_to_string(css_entry_path)
            .with_context(|| format!("Could not read {css_entry}"))?;
        if existing.contains(import_line) {
            println!("{} {} already imports yei.css", "·".dimmed(), css_entry);
        } else {
            // Insert after `@import "tailwindcss";` if present, otherwise append
            let updated = if let Some(pos) = existing.find("@import \"tailwindcss\"") {
                let end = existing[pos..].find('\n').map(|n| pos + n + 1).unwrap_or(existing.len());
                format!("{}{}\n{}", &existing[..end], import_line, &existing[end..])
            } else {
                format!("{}\n{}\n", existing.trim_end(), import_line)
            };
            std::fs::write(css_entry_path, updated)
                .with_context(|| format!("Could not update {css_entry}"))?;
            println!("{} Added @import to {}", "✓".green(), css_entry);
        }
    } else {
        let content = format!("@import \"tailwindcss\";\n{import_line}\n");
        std::fs::write(css_entry_path, content)
            .with_context(|| format!("Could not create {css_entry}"))?;
        println!("{} Created {}", "✓".green(), css_entry);
    }

    // --- Create component output directory ---
    std::fs::create_dir_all(&output_path)
        .with_context(|| format!("Could not create {output_path}"))?;
    println!("{} Created {}/", "✓".green(), output_path);

    // --- Write yei.json ---
    write_config(&config)?;
    println!("{} Created yei.json", "✓".green());

    // --- Inject :root token defaults into the CSS entry file ---
    let root_marker = "/* yei tokens */";
    let existing_css = std::fs::read_to_string(css_entry_path)
        .with_context(|| format!("Could not read {css_entry}"))?;
    if existing_css.contains(root_marker) {
        println!("{} {} already contains yei token defaults", "·".dimmed(), css_entry);
    } else {
        let root_block = format!("\n{root_marker}\n{YEI_ROOT_TOKENS}");
        let updated = format!("{}\n{}", existing_css.trim_end(), root_block);
        std::fs::write(css_entry_path, updated)
            .with_context(|| format!("Could not update {css_entry}"))?;
        println!("{} Added token defaults to {}", "✓".green(), css_entry);
    }

    // --- Print Trunk instructions ---
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. In {}, under {}:", "Trunk.toml".cyan(), "[tools]".cyan());
    println!("       {} = {}", "tailwindcss".yellow(), "\"4.3.0\"".green());
    println!("  2. In {} {}:", "index.html".cyan(), "<head>".cyan());
    println!("       {} {} {} {}",
        "<link".dimmed(),
        "data-trunk rel=\"tailwind-css\"".yellow(),
        format!("href=\"{css_entry}\"").green(),
        "/>".dimmed());
    println!("  3. Run: {}", "trunk serve".cyan());

    Ok(())
}
