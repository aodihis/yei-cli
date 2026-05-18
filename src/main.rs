use clap::{Parser, Subcommand};
mod commands;
mod config;
mod registry;

#[derive(Parser)]
#[command(name = "yei", about = "Add Yew UI components to your project")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize yei in the current project
    Init,
    /// List available components
    List,
    /// Add one or more components
    Add {
        /// Component names, optionally with version (button or button@0.1.0)
        components: Vec<String>,
        /// Override registry version
        #[arg(long)]
        version: Option<String>,
    },
    /// List available registry versions
    Versions,
    /// Upgrade all installed components to latest
    Upgrade,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => commands::init::run().await,
        Commands::List => commands::list::run().await,
        Commands::Add { components, version } => commands::add::run(components, version).await,
        Commands::Versions => commands::versions::run().await,
        Commands::Upgrade => commands::upgrade::run().await,
    }
}
