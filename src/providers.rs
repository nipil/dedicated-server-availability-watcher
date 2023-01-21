pub mod ovh;

use std::{thread, time};

use anyhow;
use anyhow::Context;

use colored::Colorize;

use crate::notifiers;
use crate::LibError;
use crate::ProviderCheckResult;

/// Defines the common information returned by `ProviderTrait::inventory()`.
pub struct ServerInfo {
    pub reference: String,
    pub memory: String,
    pub storage: String,
    pub available: bool,
}

/// Defines the expected behaviour of every provider handler.
pub trait ProviderTrait {
    /// Gets the actual name of the provider.
    fn name(&self) -> &'static str;

    /// Prints a list of every kind of server known to the provider.
    /// By default, does not include servers which are out of stock.
    /// Set 'all' to true to include unavailable server kinds.
    fn inventory(&self, all: bool) -> Result<Vec<ServerInfo>, LibError>;

    /// Checks the given provider for availability of a specific server type.
    fn check(&self, server: &str) -> Result<bool, LibError>;
}

/// Helps create providers
pub trait ProviderFactoryTrait {
    /// Builds an instance from environment variables
    fn from_env() -> Result<Box<dyn ProviderTrait>, LibError>;
}

/// Defines the expected behaviour for building providers.
type FactoryFunc = fn() -> Result<Box<dyn ProviderTrait>, LibError>;

/// Builds a reference table of available providers.
static FACTORY: &[(&str, FactoryFunc)] = &[(ovh::OVH_NAME, ovh::Ovh::from_env)];

/// Trait to help create providers
pub struct Factory;

/// Global provider factory, based on the reference table
impl Factory {
    /// Selects the desired providers type and build it from environment variables.
    pub fn from_env_by_name(provider: &str) -> Result<Box<dyn ProviderTrait>, LibError> {
        let (_, factory) = FACTORY
            .iter()
            .find(|(name, _)| *name == provider)
            .ok_or_else(|| LibError::UnknownProvider {
                provider: provider.to_string(),
            })?;
        factory()
    }

    /// Provides a list of all known provider types.
    pub fn get_available() -> Vec<&'static str> {
        let mut names: Vec<&'static str> = FACTORY.iter().map(|&(name, _)| name).collect();
        names.sort();
        names
    }
}

// Runners

/// Utility struct to manage application execution.
/// This is included in the library so it can be tested.
pub struct Runner;

impl Runner {
    /// Prints all available providers.
    pub fn run_list() -> anyhow::Result<()> {
        println!("Available providers:");
        for provider in Factory::get_available().iter() {
            println!("- {}", provider.green());
        }
        Ok(())
    }

    /// Prints a list of every kind of server known to the provider.
    /// By default, does not include servers which are out of stock
    /// Set `all` to true to include unavailable server kinds
    pub fn run_inventory(provider_name: &str, all: bool) -> anyhow::Result<()> {
        let provider = Factory::from_env_by_name(provider_name)
            .with_context(|| format!("while setting up provider {}", provider_name))?;

        println!("Working...");
        let inventory = provider
            .inventory(all)
            .with_context(|| format!("while getting inventory for provider {}", provider_name))?;

        if inventory.is_empty() {
            println!("No servers found");
            return Ok(());
        }

        println!("Available servers:");
        for item in inventory.iter() {
            match item {
                info => {
                    println!(
                        "{} {} {}",
                        if !info.available {
                            info.reference.on_red()
                        } else {
                            info.reference.green()
                        },
                        info.memory.yellow(),
                        info.storage.blue(),
                    );
                }
            }
        }
        Ok(())
    }

    /// Checks the given provider for availability of a specific server type.
    fn check_servers(
        provider: &Box<dyn ProviderTrait>,
        servers: &Vec<String>,
        result: &mut ProviderCheckResult,
    ) -> anyhow::Result<()> {
        for server in servers.iter() {
            // FIXME: do not stop on first fail ? Or remove the iteration and delegate it to the caller (but then, should store previous state)
            if provider
                .check(server)
                .with_context(|| format!("while checking for server {}", server))?
            {
                result.available_servers.push(server.clone());
            }
        }
        Ok(())
    }

    /// Checks the given provider for availability of specific server types.
    /// - if periodic check is requested, nothing happens if there is no change
    /// - if a notifier is provided, and there are any available, a notification is sent
    pub fn run_check(
        provider_name: &str,
        servers: &Vec<String>,
        notifier: &Option<String>,
        interval: &Option<u16>,
    ) -> anyhow::Result<()> {
        // builds the provider
        let provider = &Factory::from_env_by_name(provider_name)
            .with_context(|| format!("while setting up provider {}", provider_name))?;

        // initialize notifier if any
        let notifier = &match notifier {
            Some(notifier) => Some(
                notifiers::Factory::from_env_by_name(notifier)
                    .with_context(|| format!("while setting up notifier {}", notifier))?,
            ),
            None => None,
        };

        let mut last_count = 0;
        loop {
            // initialize the output structure
            let mut result = ProviderCheckResult::new(provider_name);
            Self::check_servers(provider, servers, &mut result)
                .with_context(|| format!("while checking provider {}", provider_name))?;

            // Only when a change happens do we consider notifying of the latest result
            // FIXME: need better comparison to detect changes from "A,B" to "A,C" --> sort and store and parse again
            if result.available_servers.len() != last_count {
                last_count = result.available_servers.len();

                // notify result if necessary
                match notifier {
                    None => {
                        println!("{}", result.available_servers.join(", ").green());
                    }
                    Some(notifier) => {
                        notifier
                            .notify(&result)
                            .with_context(|| format!("while notifying {}", notifier.name()))?;
                    }
                }
            }

            match interval {
                None => {
                    // exit if a single check is requested
                    break;
                }
                Some(interval) => {
                    // otherwise, wait for the specified duration
                    thread::sleep(time::Duration::from_secs((*interval).into()));
                }
            }
        }

        Ok(())
    }
}
