pub mod ifttt;

use anyhow;
use anyhow::Context;
use colored::Colorize;

use crate::{LibError, ProviderCheckResult};

/// Defines the expected behaviour of every notifier handler.
pub trait NotifierTrait {
    /// Gets the actual name of the notifier.
    fn name(&self) -> &'static str;

    /// Sends a string as notification.
    fn notify(&self, result: &ProviderCheckResult) -> Result<(), LibError>;
    /// Does whatever is required to test the notifier.
    fn test(&self) -> Result<(), LibError>;
}

/// Defines the expected behaviour for builing the desired notifier.
pub trait NotifierFactoryTrait {
    /// Builds a notifier from environment variables.
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError>;
}

/// Trait to help create notifiers
pub struct Factory;

// TODO: extract the vec! and the match into a hashmap holding closures
// that way there is a single source of truth for notifiers
impl Factory {
    /// Selects the desired notifier type and build it from environment variables.
    pub fn from_env_by_name(notifier: &str) -> Result<Box<dyn NotifierTrait>, LibError> {
        match notifier {
            ifttt::IFTTT_WEBHOOK_NAME => ifttt::WebHook::from_env(),
            _ => Err(LibError::UnknownNotifier {
                notifier: notifier.to_string(),
            }),
        }
    }

    /// Lists all known notifier types.
    pub fn list_available() -> Vec<&'static str> {
        vec![ifttt::IFTTT_WEBHOOK_NAME]
    }
}

// Runners

/// Utility struct to manage application execution.
/// This is included in the library so it can be tested.
pub struct Runner;

impl Runner {
    /// Prints all available notifiers.
    pub fn run_list() -> anyhow::Result<()> {
        println!("Available notifiers:");
        for notifier in Factory::list_available().iter() {
            println!("- {}", notifier.green());
        }
        Ok(())
    }

    /// Tests selected notifier.
    pub fn run_test(name: &str) -> anyhow::Result<()> {
        let notifier = Factory::from_env_by_name(name)
            .with_context(|| format!("while setting up notifier {}", name))?;
        notifier
            .test()
            .with_context(|| format!("while testing notifier {}", name))?;
        println!("{}", "Notification sent".to_string().green());
        Ok(())
    }
}
