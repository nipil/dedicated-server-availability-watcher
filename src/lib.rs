pub mod notifiers;
pub mod providers;

use std::env;
use std::env::VarError;

use thiserror::Error;

/// NotifierError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum LibError {
    // technical errors
    #[error("Environment variable {name} error")]
    EnvError { name: String, source: VarError },

    #[error("Network error")]
    RequestError { source: reqwest::Error },

    #[error("API error {message}")]
    ApiError { message: String },

    // login errors
    #[error("Unknown server {server}")]
    UnknownServer { server: String },

    #[error("Unknown notifier {notifier}")]
    UnknownNotifier { notifier: String },

    #[error("Unknown provider {provider} ")]
    UnknownProvider { provider: String },
}

pub fn get_env_var(name: &str) -> Result<String, LibError> {
    env::var(name).map_err(|source| LibError::EnvError {
        name: name.to_string(),
        source,
    })
}
