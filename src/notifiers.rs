use anyhow;
use anyhow::Context;
use colored::Colorize;

use crate::{CheckResult, LibError};

/// Provides the implementation for IFTTT-Webhook notifiers
#[cfg(feature = "ifttt-webhook")]
pub mod ifttt_webhook;
/// Provides the implementation for Simple notifiers
#[cfg(feature = "simple")]
pub mod simple;

/// Provides the implementation for email notifiers
#[cfg(feature = "email")]
pub mod email;

/// Defines the expected behaviour of every notifier handler.
pub trait NotifierTrait {
    /// Gets the actual name of the notifier.
    fn name(&self) -> &'static str;

    /// Sends a string as notification.
    fn notify(&self, result: &CheckResult) -> Result<(), LibError>;

    /// Does whatever is required to test the notifier.
    fn test(&self) -> Result<(), LibError>;
}

/// Defines the expected behaviour for builing the desired notifier.
pub trait NotifierFactoryTrait {
    /// Builds a notifier from environment variables.
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError>;
}

/// Defines the expected behaviour for building notifiers.
type FactoryFunc = fn() -> Result<Box<dyn NotifierTrait>, LibError>;

/// Builds a reference table of available notifiers.
static FACTORY: &[(&str, FactoryFunc)] = &[
    #[cfg(feature = "simple-get")]
    (simple::SIMPLE_GET_NAME, simple::SimpleGet::from_env),
    #[cfg(feature = "simple-post")]
    (simple::SIMPLE_POST_NAME, simple::SimplePost::from_env),
    #[cfg(feature = "simple-put")]
    (simple::SIMPLE_PUT_NAME, simple::SimplePut::from_env),
    #[cfg(feature = "ifttt-webhook-json")]
    (
        ifttt_webhook::IFTTT_WEBHOOK_JSON_NAME,
        ifttt_webhook::WebHookJson::from_env,
    ),
    #[cfg(feature = "ifttt-webhook-values")]
    (
        ifttt_webhook::IFTTT_WEBHOOK_VALUES_NAME,
        ifttt_webhook::WebHookValues::from_env,
    ),
    #[cfg(feature = "email-sendmail")]
    (
        email::EMAIL_SENDMAIL_NAME,
        email::EmailViaSendmail::from_env,
    ),
];

/// Trait to help create notifiers.
pub struct Factory;

/// Global notifier factory, based on the reference table
impl Factory {
    /// Selects the desired notifier type and build it from environment variables.
    pub fn from_env_by_name(notifier: &str) -> Result<Box<dyn NotifierTrait>, LibError> {
        let (_, factory) = FACTORY
            .iter()
            .find(|(name, _)| *name == notifier)
            .ok_or_else(|| LibError::UnknownNotifier {
                notifier: notifier.to_string(),
            })?;
        factory()
    }

    /// Provides a list of all known notifier types.
    pub fn get_available() -> Vec<&'static str> {
        let mut names: Vec<&'static str> = FACTORY.iter().map(|&(name, _)| name).collect();
        names.sort();
        names
    }
}

// Runners: included in the library so it can be tested.

/// Implementation of the ListRunner
pub struct ListRunner;

impl ListRunner {
    /// Prints all available notifiers.
    pub fn print_list() -> anyhow::Result<()> {
        println!("Available notifiers:");
        for notifier in Factory::get_available().iter() {
            println!("- {}", notifier.green());
        }
        Ok(())
    }
}
/// Implementation of the ListRunner
pub struct TestRunner {
    notifier: Box<dyn NotifierTrait>,
}

impl TestRunner {
    /// Builds an instance
    pub fn new(notifier_name: &str) -> anyhow::Result<Self> {
        Ok(Self {
            notifier: Factory::from_env_by_name(notifier_name)
                .with_context(|| format!("while setting up notifier {notifier_name}"))?,
        })
    }

    /// Tests selected notifier.
    pub fn test(&self) -> anyhow::Result<()> {
        self.notifier
            .test()
            .with_context(|| format!("while testing notifier {}", self.notifier.name()))?;
        println!("{}", "Notification sent".to_string().green());
        Ok(())
    }
}
