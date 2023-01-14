use std::collections::HashMap;

use reqwest::blocking::Response;
use serde::Deserialize;

use crate::LibError;

use super::{NotifierFactoryTrait, NotifierTrait};

// IFTTT WEBHOOK implementation

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

/// Holds the user credentials and event identifier used with the API.
pub struct WebHook {
    event: String,
    key: String,
}

impl WebHook {
    /// Builds a new instance.
    fn new(event: &str, key: &str) -> WebHook {
        let event = event.trim();
        if event.is_empty() {
            panic!("Ifttt webhook event should not be empty");
        }
        let key = key.trim();
        if key.is_empty() {
            panic!("Ifttt webhook key shoult not be empty");
        }
        // TODO: sanitize inputs according to notifier API format spec
        WebHook {
            event: event.to_string(),
            key: key.to_string(),
        }
    }

    /// Sends the actual API request.
    // TODO: add a selector for either json or valueX api
    fn query(&self, params: HashMap<&str, &str>) -> Result<Response, LibError> {
        let url = format!(
            "https://maker.ifttt.com/trigger/{}/json/with/key/{}",
            self.event, self.key
        );
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(url)
            .json(&params)
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
}

impl NotifierFactoryTrait for WebHook {
    /// Builds a WebHook notifier from environment variables.
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError> {
        let event = crate::get_env_var(ENV_NAME_IFTTT_WEBHOOK_EVENT)?;
        let key = crate::get_env_var(ENV_NAME_IFTTT_WEBHOOK_KEY)?;
        Ok(Box::new(WebHook::new(&event, &key)))
    }
}

impl NotifierTrait for WebHook {
    /// Sends an notification using the provided data.
    fn notify(&self, content: &str) -> Result<(), LibError> {
        // build request
        let params = HashMap::from([("available", content)]);

        let response = self.query(params)?;
        Ok(())
    }

    /// Tests by sending a notification with dummy values.
    fn test(&self) -> Result<(), LibError> {
        let mut params = HashMap::new();
        params.insert("value1", "foo");
        params.insert("value2", "bar");
        params.insert("value3", "baz");
        params.insert("no", "way");

        let response = self.query(params)?;
        Ok(())
    }
}
