use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::PoolConfig;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, SmtpTransport, Tokio1Executor, Transport,
};
use rocket::fairing::AdHoc;
use serde::Deserialize;

use crate::utils::InsignoError;
use crate::InsignoConfig;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SmtpConfig {
    server: String,
    user: String,
    password: String,
}

#[cfg(test)]
pub async fn send_mail(
    _to: &str,
    _subject: &str,
    _message: &str,
    _mailer: &Mailer,
) -> Result<(), InsignoError> {
    Ok(())
}

#[cfg(not(test))]
pub async fn send_mail(
    to: &str,
    subject: &str,
    message: &str,
    mailer: &Mailer,
) -> Result<(), InsignoError> {
    let email = Message::builder()
        .from("Insigno: <insigno@mindshub.it>".parse().unwrap())
        //.reply_to("mail to reply".parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(message.to_string())
        .unwrap();
    match mailer.m.send(email).await {
        Ok(_) => Ok(()),
        Err(e) => Err(InsignoError::new_debug(500, &e.to_string())), //format!("Could not send email: {e:?}")),
    }
}

pub struct Mailer {
    pub m: AsyncSmtpTransport<Tokio1Executor>,
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Gmail stage", |rocket| async {
        let config: &InsignoConfig = rocket.state().unwrap();

        let mail_config = PoolConfig::new().min_idle(1);
        let creds = Credentials::new(
            config.smtp.user.to_string(),
            config.smtp.password.to_string(),
        );
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp.server)
            .unwrap()
            .credentials(creds)
            .pool_config(mail_config)
            .build();
        let mailer = Mailer { m: mailer };
        rocket.manage(mailer)
    })
}
