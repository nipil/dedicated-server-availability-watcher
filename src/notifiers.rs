pub mod ifttt;

use std::error::Error;

use colored::Colorize;

use crate::MyError;

pub trait NotifierTrait {
    fn notify(&self, content: &str) -> Result<(), Box<dyn Error>>;
    fn test(&self) -> Result<(), Box<dyn Error>>;
}

pub trait NotifierFactoryTrait {
    fn from_env() -> Result<Box<dyn NotifierTrait>, Box<dyn Error>>;
}

pub struct Factory;

impl Factory {
    pub fn from_env_by_name(s: &str) -> Result<Box<dyn NotifierTrait>, Box<dyn Error>> {
        match s {
            "ifttt-webhook" => ifttt::WebHook::from_env(),
            _ => Err(Box::new(MyError::new(&format!("Unknown notifier '{}'", s)))),
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

    pub fn run_test(name: &str) -> Result<(), Box<dyn Error>> {
        let notifier = Factory::from_env_by_name(name)?;
        notifier.test()?;
        Ok(())
    }
}
