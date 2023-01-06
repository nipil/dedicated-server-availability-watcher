pub mod ifttt;

use colored::Colorize;

use crate::LibError;

pub trait NotifierTrait {
    fn notify(&self, content: &str) -> Result<(), LibError>;
    fn test(&self) -> Result<(), LibError>;
}

pub trait NotifierFactoryTrait {
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError>;
}

pub struct Factory;

impl Factory {
    pub fn from_env_by_name(notifier: &str) -> Result<Box<dyn NotifierTrait>, LibError> {
        match notifier {
            "ifttt-webhook" => ifttt::WebHook::from_env(),
            _ => Err(LibError::UnknownNotifier {
                notifier: notifier.to_string(),
            }),
        }
    }

    pub fn list_available() -> Vec<&'static str> {
        vec!["ifttt-webhook"]
    }
}

// Runners

pub struct Runner;

impl Runner {
    pub fn run_list() {
        println!("Available notifiers:");
        for notifier in Factory::list_available().iter() {
            println!("- {}", notifier.green());
        }
    }

    pub fn run_test(name: &str) -> Result<(), LibError> {
        let notifier = Factory::from_env_by_name(name)?;
        notifier.test()?;
        println!("{}", "Notification sent".to_string().green());
        Ok(())
    }
}
