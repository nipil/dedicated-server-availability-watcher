pub mod notifiers;
pub mod providers;

use std::env;
use std::env::VarError;

use thiserror::Error;

/// NotifierError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum LibError {
    // technical errors
    /// Missing or invalid environment variable.
    #[error("Environment variable {name} error")]
    EnvError { name: String, source: VarError },

    /// Anything from DNS resolution error, to connection time out...
    #[error("Network error")]
    RequestError { source: reqwest::Error },

    /// Anything which happen on the logical request (ie. network is ok).
    #[error("API error {message}")]
    ApiError { message: String },

    // logic errors
    /// Unknown server reference.
    #[error("Unknown server {server}")]
    UnknownServer { server: String },

    // non existing handlers.
    /// Requested notifier does not exist.
    #[error("Unknown notifier {notifier}")]
    UnknownNotifier { notifier: String },

    /// Requested provider does not exist.
    #[error("Unknown provider {provider} ")]
    UnknownProvider { provider: String },
}

/// Utility function to get an environment variable by name.
pub fn get_env_var(name: &str) -> Result<String, LibError> {
    env::var(name).map_err(|source| LibError::EnvError {
        name: name.to_string(),
        source,
    })
}
