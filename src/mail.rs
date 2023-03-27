use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::Deserialize;

use crate::utils::InsignoError;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SmtpConfig {
    server: String,
    user: String,
    password: String,
}

#[cfg(test)]
pub fn send_mail(
    _to: &str,
    _subject: &str,
    _message: &str,
    _config: &SmtpConfig,
) -> Result<(), InsignoError> {
    Ok(())
}

#[cfg(not(test))]

pub fn send_mail(
    to: &str,
    subject: &str,
    message: &str,
    config: &SmtpConfig,
) -> Result<(), InsignoError> {
    let email = Message::builder()
        .from("Insigno: <insigno@mindshub.it>".parse().unwrap())
        //.reply_to("mail to reply".parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(message.to_string())
        .unwrap();

    let creds = Credentials::new(config.user.to_owned(), config.password.to_owned());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&config.server)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => Ok(()),
        Err(e) => Err(InsignoError::new_debug(500, &e.to_string())), //format!("Could not send email: {e:?}")),
    }
}
