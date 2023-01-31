// TODO: #![deny(missing_docs)]
// TODO: #[deny(missing_doc_code_examples)]
//! This crate provides implementation and structure to query cloud 'providers'
//! for dedicated servers inventory and availability, building `CheckResult`.
//! It provides implementations to 'notify' about theses results, or their
//! change compared to previous invocation.
//! 
//! See modules implementations for available handlers.

/// Provides the implementation for CheckResult notifiers
pub mod notifiers;
/// Provides the implementation for CheckResult providers
pub mod providers;
/// Provides the implementation to store CheckResult hashes
/// This is not built as a feature that could be removed, as
/// it is at the core of the differential notification scheme.
pub mod storage;

use serde::Serialize;
use std::{env, io};
use thiserror::Error;

/// NotifierError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum LibError {
    /// input/output errors
    #[error("Input/output error")]
    // FIXME: faire marcher le #from : IOError(#[from] io::Error),
    IOError { source: io::Error },

    /// Missing or empty environment variable.
    #[error("Environment variable `{name}` error")]
    EnvError { name: String, source: env::VarError },

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

/// Utility function to get an environment variable by name and trim it
pub fn get_env_var(name: &str) -> Result<String, LibError> {
    env::var(name)
        .map(|text| text.trim().to_string())
        .map_err(|source| LibError::EnvError {
            name: name.to_string(),
            source,
        })
}

/// Same as above, but as an option instead of an result
pub fn get_env_var_option(name: &str) -> Option<String> {
    get_env_var(name).map_or_else(|_| None, |o| Some(o))
}

/// Same as above, but provides a default value instead
pub fn get_env_var_default(name: &str, default: &str) -> String {
    get_env_var_option(name).unwrap_or(default.to_string())
}

/// Splits a CSV string into tokens, and verify that no token is empty
pub fn tokenize_optional_csv_str(csv: &Option<String>) -> Result<Vec<String>, LibError> {
    Ok(match csv {
        Some(csv) => {
            // split and trim each token
            let result: Vec<String> = csv.split(',').map(|s| s.trim().to_string()).collect();
            // verify that no token is empty
            if result.iter().find(|i| i.is_empty()).is_some() {
                return Err(LibError::ValueError {
                    name: "found empty token in comma separated string".into(),
                    value: csv.into(),
                });
            }
            result
        }
        None => Vec::new(),
    })
}

/// CheckResult holds the data between providers and notifiers :
/// - `provider::check` is the data source
/// - `notifier::notify` is the data sink
#[derive(PartialEq, Serialize)]
pub struct CheckResult {
    pub provider_name: String,
    pub available_servers: Vec<String>,
}

impl CheckResult {
    /// Builds an instance with no specific sanitization
    fn new(provider_name: &str) -> Self {
        Self {
            provider_name: provider_name.to_string(),
            available_servers: Vec::<String>::new(),
        }
    }

    /// Builds an instance with dummy values for testing
    fn get_dummy() -> CheckResult {
        let mut result = CheckResult::new("dummy_provider");
        result.available_servers.extend(vec![
            "foo_server".into(),
            "bar_server".into(),
            "baz_server".into(),
        ]);
        result
    }

    // Serializes to json
    fn to_json(&self) -> Result<String, LibError> {
        serde_json::to_string(&self).map_err(|source| LibError::JsonError { source })
    }
}
