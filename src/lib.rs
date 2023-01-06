pub mod notifiers;
pub mod providers;

use std::env::VarError;

use thiserror::Error;

/// NotifierError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum LibError {
    // technical errors
    #[error("Environment variable error")]
    EnvError { source: VarError },

    #[error("Network error")]
    RequestError { source: reqwest::Error },

    #[error("API error {message}")]
    ApiError { message: String },

    // login errors
    #[error("Unknown server {server}")]
    UnknownServer { server: String },

    #[error("Unknown notifier {notifier}")]
    UnknownNotifier { notifier: String },

    #[error("Unknown provider")]
    UnknownProvider { provider: String },
}
