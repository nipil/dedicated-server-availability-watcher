pub mod ovh;

use std::{thread, time};

use anyhow;
use anyhow::Context;

use colored::Colorize;

use crate::notifiers;
use crate::LibError;

pub struct ServerInfo {
    pub reference: String,
    pub memory: String,
    pub storage: String,
    pub available: bool,
}

pub trait ProviderTrait {
    // TODO: add "name" class fn
    fn inventory(&self, all: bool) -> Result<Vec<ServerInfo>, LibError>;
    fn check(&self, server: &str) -> Result<bool, LibError>;
}

pub trait ProviderFactoryTrait {
    fn from_env() -> Result<Box<dyn ProviderTrait>, LibError>;
}

pub struct Factory;

impl Factory {
    fn from_env_by_name(provider: &str) -> Result<Box<dyn ProviderTrait>, LibError> {
        match provider {
            "ovh" => ovh::Ovh::from_env(),
            _ => Err(LibError::UnknownProvider {
                provider: provider.to_string(),
            }),
        }
    }

    fn list_available() -> Vec<&'static str> {
        vec!["ovh"]
    }
}

// Runners
pub struct Runner;

impl Runner {
    pub fn run_list() -> anyhow::Result<()> {
        println!("Available providers:");
        for provider in Factory::list_available().iter() {
            println!("- {}", provider.green());
        }
        Ok(())
    }

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

    fn check_servers(
        provider: &Box<dyn ProviderTrait>,
        servers: &Vec<String>,
    ) -> anyhow::Result<Vec<String>> {
        let mut available = Vec::new();
        for server in servers.iter() {
            // TODO: do not stop on first fail ?
            if provider
                .check(server)
                .with_context(|| format!("while checking for server {}", server))?
            {
                available.push(server.clone());
            }
        }
        Ok(available)
    }

    pub fn run_check(
        provider_name: &str,
        servers: &Vec<String>,
        notifier: &Option<String>,
        interval: &Option<u16>,
    ) -> anyhow::Result<()> {
        let provider = &Factory::from_env_by_name(provider_name)
            .with_context(|| format!("while setting up provider {}", provider_name))?;

        let notifier = &match notifier {
            Some(notifier) => Some(
                notifiers::Factory::from_env_by_name(notifier)
                    .with_context(|| format!("while setting up notifier {}", notifier))?,
            ),
            None => None,
        };

        let mut last_count = 0;
        loop {
            let available = Self::check_servers(provider, servers)
                .with_context(|| format!("while checking provider {}", provider_name))?;
            if available.len() != last_count {
                last_count = available.len();

                // build result
                let result = if !available.is_empty() {
                    available.join(", ")
                } else {
                    "".to_string()
                };

                // notify result
                match notifier {
                    None => {
                        println!("{}", result.green());
                    }
                    Some(notifier) => {
                        // TODO: add notifier name
                        notifier.notify(&result).context("while notifying")?;
                    }
                }
            }

            if interval.is_none() {
                break;
            }

            thread::sleep(time::Duration::from_secs(interval.unwrap().into()));
        }

        Ok(())
    }
}
