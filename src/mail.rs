
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::Deserialize;


#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SmtpConfig {
    server: String,
    user: String,
    password: String,
}

pub fn send_mail(to: &str, subject: &str, message: &str, config: &SmtpConfig)-> Result<(), String>{
    let email = Message::builder()
        .from("MAIL FROM: <insigno@mindshub.it>".parse().unwrap())
        //.reply_to("mail to reply".parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
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
        Err(e) => Err(format!("Could not send email: {e:?}")),
    }
}