use clap::{Parser, Subcommand};
use dedicated_server_availability_watcher::{notifiers, providers};
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // main commands
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

    // Test server type availability
    Check {
        /// Provider
        provider: String,

        /// List of server types
        #[arg(required = true)]
        servers: Vec<String>,

        /// Optional notify handler
        #[arg(short, long)]
        notifier: Option<String>,

        /// Check periodically (in seconds)
        #[arg(short, long)]
        interval: Option<u16>,
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

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        // Notifier actions
        Commands::Notifier { subcommand } => match subcommand {
            None => notifiers::Runner::run_list(),

            Some(sub) => match sub {
                NotifierCommands::List {} => notifiers::Runner::run_list(),

                NotifierCommands::Test { notifier } => notifiers::Runner::run_test(notifier)?,
            },
        },

        // Provider actions
        Commands::Provider { subcommand } => match subcommand {
            None => providers::Runner::run_list(),

            Some(sub) => match sub {
                ProviderCommands::List {} => providers::Runner::run_list(),

                ProviderCommands::Inventory { provider, all } => {
                    providers::Runner::run_inventory(provider, *all)?;
                }

                ProviderCommands::Check {
                    provider,
                    servers,
                    notifier,
                    interval,
                } => providers::Runner::run_check(provider, servers, notifier, interval)?,
            },
        },
    }

    Ok(())
}
