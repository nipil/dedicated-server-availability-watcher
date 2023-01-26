pub mod ovh;
pub mod scaleway;

use std::{thread, time};

use anyhow;
use anyhow::Context;

use colored::Colorize;

use crate::notifiers;
use crate::notifiers::NotifierTrait;
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
static FACTORY: &[(&str, FactoryFunc)] = &[
    (ovh::OVH_NAME, ovh::Ovh::from_env),
    (scaleway::SCALEWAY_NAME, scaleway::Scaleway::from_env),
];

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

    /// Builds an actual notifier from a notifier name
    fn build_provider(name: &str) -> anyhow::Result<Box<dyn ProviderTrait>> {
        Ok(Factory::from_env_by_name(name)
            .with_context(|| format!("while setting up provider {name}"))?)
    }

    /// Builds an actual notifier from a notifier name
    fn build_notifier(name: &Option<String>) -> anyhow::Result<Option<Box<dyn NotifierTrait>>> {
        Ok(match name {
            None => None,
            Some(notifier) => Some(
                notifiers::Factory::from_env_by_name(notifier)
                    .with_context(|| format!("while setting up notifier {notifier}"))?,
            ),
        })
    }

    /// Builds an actual notifier from a notifier name
    fn notify_result(
        notifier: &Option<Box<dyn NotifierTrait>>,
        result: &ProviderCheckResult,
    ) -> anyhow::Result<()> {
        match notifier {
            None => {
                println!("{}", result.available_servers.join(", ").green());
            }
            Some(notifier) => {
                notifier.notify(&result).with_context(|| {
                    format!("while notifying results through {}", notifier.name())
                })?;
            }
        }
        Ok(())
    }

    /// Prints a list of every kind of server known to the provider.
    /// By default, does not include servers which are out of stock
    /// Set `all` to true to include unavailable server kinds
    pub fn run_inventory(provider_name: &str, all: bool) -> anyhow::Result<()> {
        let provider = Factory::from_env_by_name(provider_name)
            .with_context(|| format!("while setting up provider {provider_name}"))?;

        println!("Working...");
        let inventory = provider
            .inventory(all)
            .with_context(|| format!("while getting inventory for provider {provider_name}"))?;

        if inventory.is_empty() {
            println!("No servers found");
            return Ok(());
        }

        println!("Known servers:");
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
            if provider
                .check(server)
                .with_context(|| format!("while checking for server {server}"))?
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
        notifier_name: &Option<String>,
    ) -> anyhow::Result<()> {
        let provider = Self::build_provider(provider_name)?;
        let notifier = Self::build_notifier(notifier_name)?;
        let mut result = ProviderCheckResult::new(provider_name);

        Self::check_servers(&provider, servers, &mut result)
            .with_context(|| format!("while checking provider {provider_name}"))?;

        Self::notify_result(&notifier, &result)?;
        Ok(())
    }

    /// Checks the given provider for availability of specific server types.
    /// - if periodic check is requested, nothing happens if there is no change
    /// - if a notifier is provided, and there are any available, a notification is sent
    #[cfg(feature = "check_interval")]
    pub fn run_check_interval(
        provider_name: &str,
        servers: &Vec<String>,
        notifier_name: &Option<String>,
        interval: &Option<u16>,
    ) -> anyhow::Result<()> {
        let provider = Self::build_provider(provider_name)?;
        let notifier = Self::build_notifier(notifier_name)?;
        let mut last = ProviderCheckResult::new(provider_name);

        // FIXME: ignore errors while in loop ?
        loop {
            // populate results
            let mut current = ProviderCheckResult::new(provider_name);
            Self::check_servers(&provider, servers, &mut current)
                .with_context(|| format!("while checking provider {provider_name}"))?;

            // compare, notify and age result
            if current != last {
                Self::notify_result(&notifier, &current)?;
                last = current;
            }

            // continue if necessary
            match interval {
                None => break,
                Some(interval) => thread::sleep(time::Duration::from_secs((*interval).into())),
            }
        }
        Ok(())
    }
}
