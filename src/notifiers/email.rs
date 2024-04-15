use lettre::message::Mailbox;
use lettre::SendmailTransport;
use lettre::{Message, Transport};

use crate::LibError::EmailError;
use crate::{CheckResult, LibError};

use super::{NotifierFactoryTrait, NotifierTrait};

/// Common environment variable to select the custom URL.
const ENV_EMAIL_FROM: &str = "EMAIL_FROM";
const ENV_EMAIL_TO: &str = "EMAIL_TO";

/// Common functions
fn mailbox_from_string(mailbox: &str) -> Result<Mailbox, LibError> {
    mailbox.parse::<Mailbox>().map_err(|e| EmailError {
        message: format!("{e} in `{mailbox}`"),
    })
}

/// Get a destination mailbox from the environment
fn env_mailbox_to() -> Result<Mailbox, LibError> {
    let email = crate::get_env_var(ENV_EMAIL_TO)?;
    mailbox_from_string(&email)
}

/// Maybe get an originating mailbox from the environment
fn env_mailbox_from() -> Result<Mailbox, LibError> {
    let email = crate::get_env_var(ENV_EMAIL_FROM)?;
    mailbox_from_string(&email)
}

/// Build a report message, using additional environment variables
fn env_create_message(result: &CheckResult) -> Result<Message, LibError> {
    let from = env_mailbox_from()?;
    let to = env_mailbox_to()?;
    create_message(result, to, from)
}

/// Build a report message
fn create_message(result: &CheckResult, to: Mailbox, from: Mailbox) -> Result<Message, LibError> {
    let name = &result.provider_name;
    Message::builder()
        .from(from)
        .to(to)
        .subject(format!("Server availability notification for {name}"))
        .body(result.to_string())
        .map_err(|e| EmailError {
            message: format!("{e} in ``"),
        })
}

/// Common name to identify the provider
pub const EMAIL_SENDMAIL_NAME: &str = "email-sendmail";

pub struct EmailViaSendmail {}

impl EmailViaSendmail {
    fn send(message: Message) -> Result<(), LibError> {
        SendmailTransport::new()
            .send(&message)
            .map_err(|e| EmailError {
                message: format!("{e} in `{message:?}`"),
            })
    }
}

impl NotifierFactoryTrait for EmailViaSendmail {
    /// Builds a SimpleGet notifier from environment variables.
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError> {
        Ok(Box::new(EmailViaSendmail {}))
    }
}

impl NotifierTrait for EmailViaSendmail {
    /// Gets the actual name of the notifier.
    fn name(&self) -> &'static str {
        return EMAIL_SENDMAIL_NAME;
    }

    /// Sends a notification using the provided data.
    fn notify(&self, result: &CheckResult) -> Result<(), LibError> {
        Self::send(env_create_message(result)?)
    }

    /// Tests by sending a notification with dummy values.
    fn test(&self) -> Result<(), LibError> {
        self.notify(&CheckResult::get_dummy())
    }
}
