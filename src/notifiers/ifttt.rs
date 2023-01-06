use std::collections::HashMap;

use reqwest::blocking::Response;
use serde::Deserialize;

use crate::LibError;

use super::{NotifierFactoryTrait, NotifierTrait};

// IFTTT WEBHOOK implementation

const ENV_NAME_IFTTT_WEBHOOK_EVENT: &str = "IFTTT_WEBHOOK_EVENT";
const ENV_NAME_IFTTT_WEBHOOK_KEY: &str = "IFTTT_WEBHOOK_KEY";

#[derive(Debug, Deserialize)]
struct IftttApiErrorMessage {
    message: String,
}

#[derive(Debug, Deserialize)]
struct IftttApiError {
    errors: Vec<IftttApiErrorMessage>,
}

pub struct WebHook {
    event: String,
    key: String,
}

impl WebHook {
    fn new(event: &str, key: &str) -> WebHook {
        let event = event.trim();
        if event.is_empty() {
            panic!("Ifttt webhook event should not be empty");
        }
        let key = key.trim();
        if key.is_empty() {
            panic!("Ifttt webhook key shoult not be empty");
        }
        // TODO: sanitize both inputs (single words, no space, no /)
        WebHook {
            event: event.to_string(),
            key: key.to_string(),
        }
    }

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

        return Err(LibError::ApiError {
            message: "Unknown IFTTT-WEBHOOK error".to_string(),
        });
    }
}

impl NotifierFactoryTrait for WebHook {
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError> {
        // TODO: more explicit error message when missing
        let event = crate::get_env_var(ENV_NAME_IFTTT_WEBHOOK_EVENT)?;
        let key = crate::get_env_var(ENV_NAME_IFTTT_WEBHOOK_KEY)?;
        Ok(Box::new(WebHook::new(&event, &key)))
    }
}

impl NotifierTrait for WebHook {
    fn notify(&self, content: &str) -> Result<(), LibError> {
        // build request
        let params = HashMap::from([("available", content)]);

        let response = self.query(params)?;
        Ok(())
    }

    fn test(&self) -> Result<(), LibError> {
        let mut params = HashMap::new();
        params.insert("value1", "foo");
        params.insert("value2", "bar");
        params.insert("value3", "baz");
        params.insert("foo", "bar");

        let response = self.query(params)?;
        Ok(())
    }
}

// use colored::Colorize;
// if response.status().is_success() {
//     println!("{}: Request sent.", "Success".green());
// } else if response.status().is_client_error() {
//     let response: IftttApiError = response
//         .json()
//         .map_err(|source| LibError::NetworkError { source })?;
//     let messages = response
//         .errors
//         .iter()
//         .map(|e| e.message.clone())
//         .collect::<Vec<String>>()
//         .join(" / ");
//     println!("{}: {}", "Failure".red(), messages);
// } else {
//     println!("{}: code {}", "Unknown".blue(), response.status());
// }
