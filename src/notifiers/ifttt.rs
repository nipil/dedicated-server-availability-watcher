use std::collections::HashMap;

use reqwest::blocking::Response;
use serde::Deserialize;

use crate::{LibError, ProviderCheckResult};

use super::{NotifierFactoryTrait, NotifierTrait};

// IFTTT WEBHOOK implementations

/// Names to identify the providers
pub const IFTTT_WEBHOOK_JSON_NAME: &str = "ifttt-webhook-json";
pub const IFTTT_WEBHOOK_VALUES_NAME: &str = "ifttt-webhook-values";

/// Common environment variable to select the webhook event.
const ENV_NAME_IFTTT_WEBHOOK_EVENT: &str = "IFTTT_WEBHOOK_EVENT";

/// Common environment variable to input the user API KEY.
const ENV_NAME_IFTTT_WEBHOOK_KEY: &str = "IFTTT_WEBHOOK_KEY";

/// Used for API result deserialisation.
#[derive(Debug, Deserialize)]
struct IftttApiErrorMessage {
    message: String,
}

/// Used for API result deserialisation.
#[derive(Debug, Deserialize)]
struct IftttApiError {
    errors: Vec<IftttApiErrorMessage>,
}

/// Builds dummy results for testing
fn get_dummy_provider_check_result() -> ProviderCheckResult {
    let mut result = ProviderCheckResult::new("test_provider");
    result
        .available_servers
        .extend(vec!["foo".into(), "bar".into(), "baz".into()]);
    result
}

/// Holds the configuration for the API call
struct WebHookParameters {
    event: String,
    key: String,
}

impl WebHookParameters {
    /// Builds an instance from environment variables.
    fn from_env() -> Result<Self, LibError> {
        let event = crate::get_env_var(ENV_NAME_IFTTT_WEBHOOK_EVENT)?;
        let key = crate::get_env_var(ENV_NAME_IFTTT_WEBHOOK_KEY)?;
        Ok(Self::new(&event, &key)?)
    }

    /// Builds a new instance, attempting to sanitize inputs
    fn new(event: &str, key: &str) -> Result<Self, LibError> {
        // Could not sanitize IFTTT input better, as they don't even follow their own spec:
        // webhook even says to use only letters, numbers and underscored, but it actually
        // allows - and # ... So i do not even try to sanitize.
        let event = event.trim().to_string();
        if event.is_empty() {
            return Err(LibError::ValueError {
                name: "ifttt webhook event".into(),
                value: event,
            });
        }

        // Could not sanitize IFTTT input better, as i have not found their key spec:
        // it seems to be 22 character of letters and numbers, but why risk a future
        // locking false positive trigger ? So again, i will not even try.
        let key = key.trim().to_string();
        if key.is_empty() {
            return Err(LibError::ValueError {
                name: "ifttt webhook key".into(),
                value: key,
            });
        }

        Ok(Self { event, key })
    }
}

/// Posts a request and handle Ifttt-Webhook specific errors
fn post(url: &str, body: &str) -> Result<Response, LibError> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(url)
        .body(body.to_string())
        .send()
        .map_err(|source| LibError::RequestError { source })?;

    if response.status().is_success() {
        return Ok(response);
    }

    // Handles known errors.
    if response.status().is_client_error() {
        let response: IftttApiError = response
            .json()
            .map_err(|source| LibError::RequestError { source })?;

        let messages = response
            .errors
            .iter()
            .map(|e| e.message.clone())
            .collect::<Vec<String>>()
            .join(" / ");

        return Err(LibError::ApiError {
            message: format!("Error during IFTTT-WEBHOOK query: {}", messages),
        });
    }

    // Unhandled unknown errors.
    return Err(LibError::ApiError {
        message: "Unknown IFTTT-WEBHOOK error".to_string(),
    });
}

/// Holds the user credentials and event identifier used with the API.
pub struct WebHookJson {
    url: String,
}

impl WebHookJson {
    /// Create an instance.
    fn new(parameters: &WebHookParameters) -> Self {
        let url = format!(
            // Builds ifttt 'json' URL.
            // - the first placeholder is for the event name
            // - the second placeholder is for the user's key
            "https://maker.ifttt.com/trigger/{}/json/with/key/{}",
            parameters.event, parameters.key
        );
        Self { url }
    }
}

impl NotifierFactoryTrait for WebHookJson {
    /// Builds a WebHook 'json' notifier from environment variables.
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError> {
        let parameters = WebHookParameters::from_env()?;
        Ok(Box::new(Self::new(&parameters)))
    }
}

impl NotifierTrait for WebHookJson {
    /// Gets the actual name of the notifier.
    fn name(&self) -> &'static str {
        return IFTTT_WEBHOOK_JSON_NAME;
    }

    /// Sends an notification using the provided data.
    fn notify(&self, result: &ProviderCheckResult) -> Result<(), LibError> {
        let body = result.to_json()?;
        let response = post(&self.url, &body)?;
        Ok(())
    }

    /// Tests by sending a notification with dummy values.
    fn test(&self) -> Result<(), LibError> {
        self.notify(&get_dummy_provider_check_result())
    }
}

/// Holds the user credentials and event identifier used with the API.
pub struct WebHookValues {
    url: String,
}

impl WebHookValues {
    /// Create an instance.
    fn new(parameters: &WebHookParameters) -> Self {
        let url = format!(
            // Builds ifttt 'value' URL.
            // - the first placeholder is for the event name
            // - the second placeholder is for the user's key
            "https://maker.ifttt.com/trigger/{}/with/key/{}",
            parameters.event, parameters.key
        );
        Self { url }
    }

    /// Builds a POST body from query parameters
    fn build_body(
        &self,
        provider_tag: &str,
        server_tag: &str,
        result: &ProviderCheckResult,
    ) -> Result<String, LibError> {
        let joined = result.available_servers.join(",");
        let mut params = HashMap::new();
        params.insert(provider_tag, &result.provider_name);
        params.insert(server_tag, &joined);
        serde_json::to_string(&params).map_err(|source| LibError::JsonError { source })
    }
}

impl NotifierFactoryTrait for WebHookValues {
    /// Builds a WebHook 'values' notifier from environment variables.
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError> {
        let parameters = WebHookParameters::from_env()?;
        Ok(Box::new(Self::new(&parameters)))
    }
}

impl NotifierTrait for WebHookValues {
    /// Gets the actual name of the notifier.
    fn name(&self) -> &'static str {
        return IFTTT_WEBHOOK_VALUES_NAME;
    }

    /// Sends an notification using the provided data.
    fn notify(&self, result: &ProviderCheckResult) -> Result<(), LibError> {
        let body = self.build_body("value1", "value2", result)?;
        let response = post(&self.url, &body)?;
        Ok(())
    }

    /// Tests by sending a notification with dummy values.
    fn test(&self) -> Result<(), LibError> {
        self.notify(&get_dummy_provider_check_result())
    }
}
