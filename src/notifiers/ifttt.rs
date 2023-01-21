use std::collections::HashMap;

use reqwest::blocking::Response;
use serde::Deserialize;

use crate::{LibError, ProviderCheckResult};

use super::{NotifierFactoryTrait, NotifierTrait};

// IFTTT WEBHOOK implementation

/// Common name to identify the provider
pub const IFTTT_WEBHOOK_NAME: &str = "ifttt-webhook";

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
    fn new(variant: &str, event: &str, key: &str) -> Result<Self, LibError> {
        let variant = variant.trim();
        let variant = match variant {
            "value" => WebHookVariant::Value,
            "json" => WebHookVariant::Json,
            _ => {
                return Err(LibError::ValueError {
                    name: "ifttt webhook variant".into(),
                    value: variant.into(),
                });
            }
        };

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

        Ok(WebHook {
            event,
            key,
            variant,
        })
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
                WebHookVariant::Json => "/json",
            },
            self.key
        )
    }

    /// Sends the actual API request.
    fn query(&self, body: &str) -> Result<Response, LibError> {
        let url = self.get_url();
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
}

impl NotifierFactoryTrait for WebHook {
    /// Builds a WebHook notifier from environment variables.
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError> {
        let variant = crate::get_env_var(ENV_NAME_IFTTT_WEBHOOK_VARIANT)?;
        let event = crate::get_env_var(ENV_NAME_IFTTT_WEBHOOK_EVENT)?;
        let key = crate::get_env_var(ENV_NAME_IFTTT_WEBHOOK_KEY)?;
        Ok(Box::new(WebHook::new(&variant, &event, &key)?))
    }
}

impl NotifierTrait for WebHook {
    /// Gets the actual name of the notifier.
    fn name(&self) -> &'static str {
        return IFTTT_WEBHOOK_NAME;
    }

    /// Sends an notification using the provided data.
    fn notify(&self, result: &ProviderCheckResult) -> Result<(), LibError> {
        // this is outside the match so that it lives beyond
        // the inner statement and can be borrowed by query
        let joined = result.available_servers.join(", ");

        // handles variant
        let body = match self.variant {
            WebHookVariant::Value => {
                let mut params = HashMap::new();
                params.insert("value1", &result.provider_name);
                let value2 = result.available_servers.join(", ");
                params.insert("value2", &value2);
                serde_json::to_string(&params).map_err(|source| LibError::JsonError { source })?
            }
            WebHookVariant::Json => result.to_json()?,
        };

        let response = self.query(&body)?;
        Ok(())
    }

    /// Tests by sending a notification with dummy values.
    fn test(&self) -> Result<(), LibError> {
        let mut test_result = ProviderCheckResult::new("test_provider");
        test_result
            .available_servers
            .extend(vec!["foo".into(), "bar".into(), "baz".into()]);
        self.notify(&test_result)
    }
}
