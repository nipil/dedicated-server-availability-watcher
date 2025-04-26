use super::{NotifierFactoryTrait, NotifierTrait};
use crate::LibError::{EmailError, ValueError};
use crate::{CheckResult, LibError};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::response::Severity;
use lettre::transport::smtp::{SMTP_PORT, SUBMISSIONS_PORT, SUBMISSION_PORT};
use lettre::SendmailTransport;
use lettre::{Message, SmtpTransport, Transport};

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
    /// Builds a EmailViaSendmail notifier from environment variables.
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError> {
        Ok(Box::new(EmailViaSendmail {}))
    }
}

impl NotifierTrait for EmailViaSendmail {
    /// Gets the actual name of the notifier.
    fn name(&self) -> &'static str {
        EMAIL_SENDMAIL_NAME
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

/// Common environment variable to select the custom URL.
const ENV_EMAIL_SMTP_HOST: &str = "EMAIL_SMTP_HOST";
const ENV_EMAIL_SMTP_PORT: &str = "EMAIL_SMTP_PORT";
const ENV_EMAIL_SMTP_USER: &str = "EMAIL_SMTP_USER";
const ENV_EMAIL_SMTP_PASSWORD: &str = "EMAIL_SMTP_PASSWORD";

/// Common name to identify the provider
pub const EMAIL_SMTP_NAME: &str = "email-smtp";

pub struct EmailViaSmtp {
    host: String,
    port: u16,
    user: String,
    password: String,
}

impl EmailViaSmtp {
    fn send(&self, message: Message) -> Result<(), LibError> {
        let builder = match self.port {
            SUBMISSIONS_PORT => SmtpTransport::relay(self.host.as_str()),
            SMTP_PORT | SUBMISSION_PORT => SmtpTransport::starttls_relay(self.host.as_str()),
            _ => {
                return Err(ValueError {
                    name: ENV_EMAIL_SMTP_PORT.to_string(),
                    value: format!("Unknown STARTTLS or TLS from port {}", self.port),
                })
            }
        }
        .map_err(|e| EmailError {
            message: format!("Error when creating SMTP transport : {e}"),
        })?;

        let sender = builder
            .port(self.port)
            .credentials(Credentials::new(self.user.clone(), self.password.clone()))
            .authentication(vec![Mechanism::Plain, Mechanism::Login, Mechanism::Xoauth2])
            .build();

        let response = sender.send(&message).map_err(|e| EmailError {
            message: format!("Smtp error when sending email message : {e}"),
        })?;

        let messages = response.message().fold(String::new(), |mut a, b| {
            a.reserve(b.len() + 1);
            a.push_str(b);
            a.push_str("\n");
            a
        });
        let messages = messages.trim_end();

        match response.code().severity {
            Severity::PositiveCompletion | Severity::PositiveIntermediate => Ok(()),
            Severity::TransientNegativeCompletion => Err(EmailError {
                message: format!("Negative smtp TRANSIENT response : {messages}"),
            }),
            Severity::PermanentNegativeCompletion => Err(EmailError {
                message: format!("Negative smtp PERMANENT response : {messages}"),
            }),
        }
    }
}

impl NotifierFactoryTrait for EmailViaSmtp {
    /// Builds a notifier from environment variables.
    fn from_env() -> Result<Box<dyn NotifierTrait>, LibError> {
        let host = crate::get_env_var(ENV_EMAIL_SMTP_HOST)?;
        let port = crate::get_env_var(ENV_EMAIL_SMTP_PORT)?;
        let port = port.parse().map_err(|e| ValueError {
            name: ENV_EMAIL_SMTP_PORT.to_string(),
            value: format!("{e}: {port}"),
        })?;
        let user = crate::get_env_var(ENV_EMAIL_SMTP_USER)?;
        let password = crate::get_env_var(ENV_EMAIL_SMTP_PASSWORD)?;
        Ok(Box::new(EmailViaSmtp {
            host,
            port,
            user,
            password,
        }))
    }
}

impl NotifierTrait for EmailViaSmtp {
    /// Gets the actual name of the notifier.
    fn name(&self) -> &'static str {
        EMAIL_SENDMAIL_NAME
    }

    /// Sends a notification using the provided data.
    fn notify(&self, result: &CheckResult) -> Result<(), LibError> {
        self.send(env_create_message(result)?)
    }

    /// Tests by sending a notification with dummy values.
    fn test(&self) -> Result<(), LibError> {
        self.notify(&CheckResult::get_dummy())
    }
}
