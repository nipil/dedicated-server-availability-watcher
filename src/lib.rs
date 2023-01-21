pub mod notifiers;
pub mod providers;

use std::env;
use std::env::VarError;

use serde::Serialize;
use thiserror::Error;

/// NotifierError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum LibError {
    // technical errors
    /// Missing or empty environment variable.
    #[error("Environment variable `{name}` error")]
    EnvError { name: String, source: VarError },

    /// Invalid value errors
    #[error("Invalid variable `{name}` error with value `{value}`")]
    ValueError { name: String, value: String },

    /// Anything from DNS resolution error, to connection time out...
    #[error("Network error")]
    RequestError { source: reqwest::Error },

    /// Anything which happen on the logical request (ie. network is ok).
    #[error("API error `{message}`")]
    ApiError { message: String },

    /// Anything which happen upon json serialization/deserialization.
    #[error("Json error")]
    JsonError { source: serde_json::Error },

    // logic errors
    /// Unknown server reference.
    #[error("Unknown server `{server}`")]
    UnknownServer { server: String },

    // non existing handlers.
    /// Requested notifier does not exist.
    #[error("Unknown notifier `{notifier}`")]
    UnknownNotifier { notifier: String },

    /// Requested provider does not exist.
    #[error("Unknown provider `{provider}` ")]
    UnknownProvider { provider: String },
}

/// Utility function to get an environment variable by name.
pub fn get_env_var(name: &str) -> Result<String, LibError> {
    env::var(name).map_err(|source| LibError::EnvError {
        name: name.to_string(),
        source,
    })
}

/// ProviderCheckResult holds the data between providers and notifiers :
/// - `provider::check` is the data source
/// - `notifier::notify` is the data sink
#[derive(Debug, Serialize)]
pub struct ProviderCheckResult {
    pub provider_name: String,
    pub available_servers: Vec<String>,
}

impl ProviderCheckResult {
    fn new(provider_name: &str) -> Self {
        ProviderCheckResult {
            provider_name: provider_name.to_string(),
            available_servers: Vec::<String>::new(),
        }
    }

    // Serializes to json
    fn to_json(&self) -> Result<String, LibError> {
        serde_json::to_string(&self).map_err(|source| LibError::JsonError { source })
    }
}
