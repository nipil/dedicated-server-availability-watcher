use std::collections::HashMap;

use reqwest::blocking::{Client, RequestBuilder};

use crate::{LibError, ProviderCheckResult};

use super::{NotifierFactoryTrait, NotifierTrait};

// SIMPLE implementation (get, post, put)

/// Common name to identify the provider
pub const SIMPLE_GET_NAME: &str = "simple-get";
pub const SIMPLE_POST_NAME: &str = "simple-post";
pub const SIMPLE_PUT_NAME: &str = "simple-put";

/// Common environment variable to select the custom URL.
const ENV_SIMPLE_URL: &str = "SIMPLE_URL";

/// Environment variable to optionally select the name of the query parameter for the GET request.
const ENV_SIMPLE_GET_PARAM_NAME_PROVIDER: &str = "SIMPLE_GET_PARAM_NAME_PROVIDER";
const ENV_SIMPLE_GET_PARAM_NAME_SERVERS: &str = "SIMPLE_GET_PARAM_NAME_SERVERS";

/// Utility function to handle the execution of the request
fn send_request(builder: RequestBuilder, notifier_name: &str) -> Result<(), LibError> {
    let response = builder
        .send()
        .map_err(|source| LibError::RequestError { source })?;

    response
        .status()
        .is_success()
        .then_some(())
        .ok_or(LibError::ApiError {
            message: format!(
                "Error {} while notifying {}: {}",
                response.status().as_str(),
                notifier_name,
                response
                    .text()
                    .map_err(|source| LibError::RequestError { source })
                    .unwrap_or_else(|error| error.to_string())
            ),
        })
}

/// Implementation of a simple GET request to a custom URL
/// It picks the URL, and the query parameter names from environment variables
/// When notifying, it provides the provider name in a parameter,
/// and a comma-separated list of server name in the other parameter
pub struct SimpleGet {
    url: String,
    param_provider: String,
    param_servers: String,
}

impl NotifierFactoryTrait for SimpleGet {
    /// Builds a SimpleGet notifier from environment variables.
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError> {
        let url = crate::get_env_var(ENV_SIMPLE_URL)?;
        let param_provider = crate::get_env_var(ENV_SIMPLE_GET_PARAM_NAME_PROVIDER)?;
        let param_servers = crate::get_env_var(ENV_SIMPLE_GET_PARAM_NAME_SERVERS)?;
        Ok(Box::new(SimpleGet {
            url,
            param_provider,
            param_servers,
        }))
    }
}

impl SimpleGet {
    /// Builds the query parameter from the structure's data
    fn build_query_parameters(&self, result: &ProviderCheckResult) -> HashMap<&String, String> {
        let joined = result.available_servers.join(", ");
        let mut params = HashMap::new();
        params.insert(&self.param_provider, result.provider_name.clone());
        params.insert(&self.param_servers, joined);
        params
    }
}

impl NotifierTrait for SimpleGet {
    /// Gets the actual name of the notifier.
    fn name(&self) -> &'static str {
        return SIMPLE_GET_NAME;
    }

    /// Sends an notification using the provided data.
    fn notify(&self, result: &ProviderCheckResult) -> Result<(), LibError> {
        let params = self.build_query_parameters(result);
        let builder = Client::new().get(&self.url).query(&params);
        send_request(builder, self.name())
    }

    /// Tests by sending a notification with dummy values.
    fn test(&self) -> Result<(), LibError> {
        self.notify(&ProviderCheckResult::get_dummy())
    }
}

/// Implementation of a simple POST request to a custom URL
/// It picks the URL, and sets the body to the json serialization of the result
pub struct SimplePost {
    url: String,
}

impl NotifierFactoryTrait for SimplePost {
    /// Builds a SimplePost notifier from environment variables.
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError> {
        let url = crate::get_env_var(ENV_SIMPLE_URL)?;
        Ok(Box::new(SimplePost { url }))
    }
}

impl NotifierTrait for SimplePost {
    /// Gets the actual name of the notifier.
    fn name(&self) -> &'static str {
        return SIMPLE_POST_NAME;
    }

    /// Sends an notification using the provided data.
    fn notify(&self, result: &ProviderCheckResult) -> Result<(), LibError> {
        let json = result.to_json()?;
        let builder = Client::new().post(&self.url).body(json);
        send_request(builder, self.name())
    }

    /// Tests by sending a notification with dummy values.
    fn test(&self) -> Result<(), LibError> {
        self.notify(&ProviderCheckResult::get_dummy())
    }
}

/// Implementation of a simple POST request to a custom URL
/// It picks the URL, and sets the body to the json serialization of the result
pub struct SimplePut {
    url: String,
}

impl NotifierFactoryTrait for SimplePut {
    /// Builds a SimplePut notifier from environment variables.
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError> {
        let url = crate::get_env_var(ENV_SIMPLE_URL)?;
        Ok(Box::new(SimplePut { url }))
    }
}

impl NotifierTrait for SimplePut {
    /// Gets the actual name of the notifier.
    fn name(&self) -> &'static str {
        return SIMPLE_PUT_NAME;
    }

    /// Sends an notification using the provided data.
    fn notify(&self, result: &ProviderCheckResult) -> Result<(), LibError> {
        let json = result.to_json()?;
        let builder = Client::new().put(&self.url).body(json);
        send_request(builder, self.name())
    }

    /// Tests by sending a notification with dummy values.
    fn test(&self) -> Result<(), LibError> {
        self.notify(&ProviderCheckResult::get_dummy())
    }
}
