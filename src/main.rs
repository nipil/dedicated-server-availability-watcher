use anyhow;
use clap::{Parser, Subcommand};
use dedicated_server_availability_watcher::{notifiers, providers};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

// CLAP command line arguments declaration

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Main commands
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// provider actions
    Provider {
        #[command(subcommand)]
        subcommand: Option<ProviderCommands>,
    },

    /// notifier actions
    Notifier {
        #[command(subcommand)]
        subcommand: Option<NotifierCommands>,
    },
}

#[derive(Subcommand)]
enum ProviderCommands {
    /// List known providers types
    List {},

    /// List known server types
    Inventory {
        /// Provider
        provider: String,

        /// List even currently unavailable types
        #[arg(short, long)]
        all: bool,
    },

    /// Checks provider for server availability
    Check {
        /// Provider
        provider: String,

        /// Storage directory (defaults to current)
        #[arg(short, long)]
        storage_dir: Option<String>,

        /// List of server types
        #[arg(required = true)]
        servers: Vec<String>,

        /// Optional notify handler
        #[arg(short, long)]
        notifier: Option<String>,
    },
}

#[derive(Subcommand)]
enum NotifierCommands {
    /// List available notifiers
    List {},

    /// Send a test notification
    Test {
        /// Notifier to test
        notifier: String,
    },
}

/// Main entrypoint, uses "clap" crate for argument handling
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    match &cli.command {
        // Notifier actions
        Commands::Notifier { subcommand } => match subcommand {
            None => notifiers::ListRunner::print_list(),

            Some(sub) => match sub {
                NotifierCommands::List {} => notifiers::ListRunner::print_list(),

                NotifierCommands::Test { notifier } => {
                    notifiers::TestRunner::new(notifier)?.test()?
                }
            },
        },

        // Provider actions
        Commands::Provider { subcommand } => match subcommand {
            None => providers::ListRunner::print_list(),

            Some(sub) => match sub {
                ProviderCommands::List {} => providers::ListRunner::print_list(),

                ProviderCommands::Inventory { provider, all } => {
                    providers::InventoryRunner::new(provider)?.list_inventory(*all)?;
                }

                ProviderCommands::Check {
                    provider,
                    servers,
                    notifier,
                    storage_dir,
                } => providers::CheckRunner::new(provider, servers, notifier, storage_dir)?
                    .check_once()?,
            },
        },
    }

    Ok(())
}
