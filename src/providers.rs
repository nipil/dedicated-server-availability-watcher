use tracing::{debug, info, instrument};

/// Provides the implementation for the "online" provider
#[cfg(feature = "online")]
pub mod online;

/// Provides the implementation for the "ovh" provider
#[cfg(feature = "ovh")]
pub mod ovh;

/// Provides the implementation for the "scaleway" provider
#[cfg(feature = "scaleway")]
pub mod scaleway;

use crate::notifiers;
use crate::notifiers::NotifierTrait;
use crate::storage::CheckResultStorage;
use crate::CheckResult;
use crate::LibError;
use crate::LibError::GenericError;
use colored::Colorize;
use std::{env, path};

/// Defines the common information returned by `ProviderTrait::inventory()`.
#[derive(Clone, Debug)]
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
    /// Set 'include_unavailable' to true to include unavailable server kinds.
    fn inventory(&self, include_unavailable: bool) -> Result<Vec<ServerInfo>, LibError>;

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
    #[cfg(feature = "online")]
    (online::ONLINE_NAME, online::Online::from_env),
    #[cfg(feature = "ovh")]
    (ovh::OVH_NAME, ovh::Ovh::from_env),
    #[cfg(feature = "scaleway")]
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
}

// Runners: included in the library so they can be tested.

/// Utility struct to manage application execution.
struct Runner;

impl Runner {
    /// Builds an actual notifier from a notifier name
    fn build_provider(name: &str) -> Result<Box<dyn ProviderTrait>, LibError> {
        Ok(Factory::from_env_by_name(name)?)
    }

    /// Builds an actual notifier from a notifier name
    fn build_notifier(name: &Option<String>) -> Result<Option<Box<dyn NotifierTrait>>, LibError> {
        Ok(match name {
            None => None,
            Some(notifier) => Some(notifiers::Factory::from_env_by_name(notifier)?),
        })
    }

    /// Builds an accessor for stored results
    fn build_storage(storage_dir: &Option<String>) -> Result<CheckResultStorage, LibError> {
        let path = match storage_dir {
            Some(dir) => path::Path::new(&dir).to_path_buf(),
            None => env::current_dir().map_err(|err| GenericError {
                message: format!("Could not get current directory : {err}"),
            })?,
        };
        Ok(CheckResultStorage::new(&path)?)
    }

    /// Notify results using provided notifier
    #[instrument(skip_all, name = "Notify result")]
    fn notify_result(
        notifier: &Option<Box<dyn NotifierTrait>>,
        result: &CheckResult,
    ) -> Result<(), LibError> {
        match notifier {
            None => {
                for srv in result.available_servers.iter() {
                    println!("{}", srv.green());
                }
            }
            Some(notifier) => {
                notifier.notify(&result)?;
            }
        }
        Ok(())
    }
}

/// An implementation for the InventoryRunner
pub struct ListRunner;

impl ListRunner {
    /// Prints all available providers.
    pub fn print_list() {
        let mut names: Vec<&'static str> = FACTORY.iter().map(|&(name, _)| name).collect();
        names.sort();
        println!("Available providers:");
        for provider in names {
            println!("- {}", provider.green());
        }
    }
}

/// An implementation for the InventoryRunner
pub struct InventoryRunner {
    provider: Box<dyn ProviderTrait>,
}

impl InventoryRunner {
    /// Builds an instance so that we do not endlessly repeat arguments
    pub fn new(provider_name: &str) -> Result<Self, LibError> {
        Ok(Self {
            provider: Runner::build_provider(provider_name)?,
        })
    }

    /// Prints a list of every kind of server known to the provider.
    /// By default, does not include servers which are out of stock
    /// Set `all` to true to include unavailable server kinds
    pub fn list_inventory(&self, all: bool) -> Result<(), LibError> {
        info!("Fetching inventory ({:?}", all);
        let inventory = self.provider.inventory(all)?;
        if inventory.is_empty() {
            return Ok(());
        }
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
}

impl Runner {}

/// An implementation for the CheckRunner
pub struct CheckRunner<'a> {
    provider: Box<dyn ProviderTrait>,
    servers: &'a Vec<String>,
    notifier: Option<Box<dyn NotifierTrait>>,
    storage: CheckResultStorage,
}

impl<'a> CheckRunner<'a> {
    /// Builds an instance so that we do not endlessly repeat arguments
    pub fn new(
        provider_name: &str,
        servers: &'a Vec<String>,
        notifier_name: &Option<String>,
        storage_dir: &'a Option<String>,
    ) -> Result<Self, LibError> {
        Ok(Self {
            provider: Runner::build_provider(provider_name)?,
            servers,
            notifier: Runner::build_notifier(notifier_name)?,
            storage: Runner::build_storage(storage_dir)?,
        })
    }

    /// Checks the given provider for availability of a specific server type.
    fn check_servers(&self, result: &mut CheckResult) -> Result<(), LibError> {
        for server in self.servers.iter() {
            if self.provider.check(server)? {
                result.available_servers.push(server.clone());
            }
        }
        Ok(())
    }

    /// Checks the given provider, compare with previous result, and notify if needed
    pub fn check_once(&self) -> Result<(), LibError> {
        let provider_name = self.provider.name();

        // get current result
        let mut latest = CheckResult::new(provider_name);
        self.check_servers(&mut latest)?;

        // do nothing more if there was no change
        if self
            .storage
            .is_equal(&provider_name, &self.servers, &latest)?
        {
            debug!("check_once: storage is equal");
            return Ok(());
        }

        // store latest
        self.storage
            .put_hash(provider_name, self.servers, &latest)?;

        // Notify of the new
        Runner::notify_result(&self.notifier, &latest)
    }
}
