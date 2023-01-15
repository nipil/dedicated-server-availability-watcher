use std::collections::HashMap;

use reqwest::blocking::Response;
use serde::Deserialize;

use crate::LibError;

use super::{NotifierFactoryTrait, NotifierTrait};

// IFTTT WEBHOOK implementation

/// Common environment variable to select the webhook input variant.
const ENV_NAME_IFTTT_WEBHOOK_VARIANT: &str = "IFTTT_WEBHOOK_VARIANT";

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

/// Variant selecting the structure of the API input.
enum WebHookVariant {
    Value,
    Json,
}

/// Holds the user credentials and event identifier used with the API.
pub struct WebHook {
    variant: WebHookVariant,
    event: String,
    key: String,
}

impl WebHook {
    /// Builds a new instance.
    fn new(variant: &str, event: &str, key: &str) -> Self {
        let variant = variant.trim();
        if variant.is_empty() {
            panic!("Ifttt webhook variant shoult not be empty");
        }
        let variant = match variant {
            "value" => WebHookVariant::Value,
            "json" => WebHookVariant::Json,
            // FIXME: return error instead of panicking
            _ => panic!("Invalid ifttt webhook variant"),
        };

        // TODO: sanitize inputs according to notifier API format spec.
        let event = event.trim().to_string();
        if event.is_empty() {
            // FIXME: return error instead of panicking
            panic!("Ifttt webhook event should not be empty");
        }
        let key = key.trim().to_string();
        if key.is_empty() {
            // FIXME: return error instead of panicking
            panic!("Ifttt webhook key shoult not be empty");
        }

        WebHook {
            event,
            key,
            variant,
        }
    }

    /// Builds ifttt URL according to selected variant.
    fn get_url(&self) -> String {
        format!(
            // Common URL to send queries to IFTTT webhook
            // - the first placeholder is for the event name
            // - the second placeholder is for the variant (empty or `/json`)
            // - the third placeholder is for the user's key
            "https://maker.ifttt.com/trigger/{}{}/with/key/{}",
            self.event,
            match self.variant {
                WebHookVariant::Value => "",
                WebHookVariant::Json => "json/",
            },
            self.key
        )
    }

    /// Sends the actual API request.
    fn query(&self, params: HashMap<&str, &str>) -> Result<Response, LibError> {
        let url = self.get_url();
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
        let variant = crate::get_env_var(ENV_NAME_IFTTT_WEBHOOK_VARIANT)?;
        let event = crate::get_env_var(ENV_NAME_IFTTT_WEBHOOK_EVENT)?;
        let key = crate::get_env_var(ENV_NAME_IFTTT_WEBHOOK_KEY)?;
        Ok(Box::new(WebHook::new(&variant, &event, &key)))
    }
}

impl NotifierTrait for WebHook {
    /// Sends an notification using the provided data.
    fn notify(&self, content: &Vec<String>) -> Result<(), LibError> {
        let mut params = HashMap::new();

        // this is outside the match so that it lives beyond
        // the inner statement and can be borrowed by query
        let joined = content.join(", ");

        // handles variant
        match self.variant {
            WebHookVariant::Value => {
                params.insert("value1", joined.as_str());
            }
            WebHookVariant::Json => {
                // FIXME: add actual json array !
                params.insert("available", joined.as_str());
            }
        }

        let response = self.query(params)?;
        Ok(())
    }

    /// Tests by sending a notification with dummy values.
    fn test(&self) -> Result<(), LibError> {
        let mut params = HashMap::new();
        match self.variant {
            WebHookVariant::Value => {
                params.insert("value1", "foo");
                params.insert("value2", "bar");
                params.insert("value3", "baz");
            }
            WebHookVariant::Json => {
                params.insert("dummy", "content");
            }
        }

        let response = self.query(params)?;
        Ok(())
    }
}
