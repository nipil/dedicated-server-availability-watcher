pub mod ovh;

use std::error::Error;
use std::{thread, time};

use colored::Colorize;

use crate::notifiers;
use crate::MyError;

pub struct ServerInfo {
    pub reference: String,
    pub memory: String,
    pub storage: String,
    pub available: bool,
}

pub trait ProviderTrait {
    // TODO: add "name" class fn
    fn inventory(&self, all: bool) -> Result<Vec<ServerInfo>, Box<dyn Error>>;
    fn check(&self, server: &str) -> Result<bool, Box<dyn Error>>;
}

pub trait ProviderFactoryTrait {
    fn from_env() -> Result<Box<dyn ProviderTrait>, Box<dyn Error>>;
}

pub struct Factory;

impl Factory {
    fn from_env_by_name(s: &str) -> Result<Box<dyn ProviderTrait>, Box<dyn Error>> {
        match s {
            "ovh" => ovh::Ovh::from_env(),
            _ => Err(Box::new(MyError::new(&format!("Unknown provider '{}'", s)))),
        }
    }

    fn list_available() -> Vec<&'static str> {
        vec!["ovh"]
    }
}

// Runners
pub struct Runner;

impl Runner {
    pub fn run_list() {
        println!("Available providers:");
        for provider in Factory::list_available().iter() {
            println!("- {}", provider.green());
        }
    }

    pub fn run_inventory(provider: &str, all: bool) -> Result<(), Box<dyn Error>> {
        let provider = Factory::from_env_by_name(provider)?;

        println!("Working...");
        let inventory = provider.inventory(all)?;

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
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut available = Vec::new();
        for server in servers.iter() {
            // TODO: do not stop on first fail ?
            if provider.check(server)? {
                available.push(server.clone());
            }
        }
        Ok(available)
    }

    pub fn run_check(
        provider: &str,
        servers: &Vec<String>,
        notifier: &Option<String>,
        interval: &Option<u16>,
    ) -> Result<(), Box<dyn Error>> {
        let provider = &Factory::from_env_by_name(provider)?;

        let notifier = &match notifier {
            Some(notifier) => Some(notifiers::Factory::from_env_by_name(notifier)?),
            None => None,
        };

        let mut last_count = 0;
        loop {
            let available = Self::check_servers(provider, servers)?;
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
                        // TODO: add provider name
                        notifier.notify(&result)?;
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
