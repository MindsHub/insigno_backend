use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use rocket::tokio::fs;

use lettre::message::header::Header;
use lettre::message::SinglePart;
use lettre::message::{header::ContentType, Attachment, Body, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::PoolConfig;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, SmtpTransport, Tokio1Executor, Transport,
};
use rocket::fairing::AdHoc;
use serde::{Deserialize, Serialize};

use crate::auth::signup::SignupInfo;
use crate::auth::user::User;
use crate::utils::InsignoError;
use crate::InsignoConfig;

#[derive(Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SmtpConfig {
    server: String,
    user: String,
    password: String,
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Gmail stage", |rocket| async {
        let config: &InsignoConfig = rocket.state().unwrap();
        let mailer = MailBuilder::new(config).await.unwrap();
        rocket.manage(mailer)
    })
}

pub struct MailBuilder {
    registration_mail_content: String,
    registration_mail_content_plain: String,

    change_password_mail_content: String,
    change_password_mail_content_plain: String,

    logo_insigno: Body,
    logo_mindshub: Body,
    logo_ala: Body,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}
impl MailBuilder {
    #[cfg(not(test))]
    async fn send(&self, message: Message) -> Result<(), Box<dyn Error>> {
        self.mailer.send(message).await?;
        Ok(())
    }

    #[cfg(test)]
    async fn send(&self, message: Message) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl MailBuilder {
    async fn new(config: &InsignoConfig) -> Result<Self, Box<dyn Error>> {
        let mail_config = PoolConfig::new().min_idle(1);
        let creds = Credentials::new(
            config.smtp.user.to_string(),
            config.smtp.password.to_string(),
        );
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp.server)
                .unwrap()
                .credentials(creds)
                .pool_config(mail_config)
                .build();

        let tmp = config.template_folder.clone();
        let tmp = PathBuf::from(config.template_folder.clone());
        let logo_insigno = fs::read(tmp.join("logo_insigno.png")).await?;
        let logo_insigno = Body::new(logo_insigno);

        let logo_ala = fs::read(tmp.join("logo_ala.png")).await?;
        let logo_ala = Body::new(logo_ala);

        let logo_mindshub = fs::read(tmp.join("logo_mindshub.png")).await?;
        let logo_mindshub = Body::new(logo_mindshub);

        let registration_mail_content_plain =
            String::from_utf8(fs::read(tmp.join("mail_account_creation_plain.txt")).await?)?;
        let registration_mail_content =
            String::from_utf8(fs::read(tmp.join("mail_account_creation.html")).await?)?;

        let change_password_mail_content_plain =
            String::from_utf8(fs::read(tmp.join("mail_change_password_plain.txt")).await?)?;
        let change_password_mail_content =
            String::from_utf8(fs::read(tmp.join("mail_change_password.html")).await?)?;
        Ok(MailBuilder {
            registration_mail_content,
            registration_mail_content_plain,

            change_password_mail_content,
            change_password_mail_content_plain,
            logo_ala,
            logo_insigno,
            logo_mindshub,
            mailer,
        })
    }

    pub async fn send_registration_mail(
        &self,
        email: &str,
        user_name: &str,
        link: &str,
    ) -> Result<(), Box<dyn Error>> {
        let plain = self
            .registration_mail_content_plain
            .replace("{user}", user_name)
            .replace("{email}", email)
            .replace("{link}", link);
        let html: String = self
            .registration_mail_content
            .replace("{user}", user_name)
            .replace("{email}", email)
            .replace("{link}", link);

        let message = Message::builder()
            .from("Insigno <insigno@mindshub.it>".parse().unwrap())
            .to(email.parse().unwrap())
            .subject("Registrazione account Insigno")
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::plain(plain))
                    .multipart(
                        MultiPart::related()
                            .singlepart(SinglePart::html(html))
                            .singlepart(
                                Attachment::new_inline(String::from("123"))
                                    .body(self.logo_insigno.clone(), "image/png".parse().unwrap()),
                            )
                            .singlepart(
                                Attachment::new_inline(String::from("124"))
                                    .body(self.logo_mindshub.clone(), "image/png".parse().unwrap()),
                            )
                            .singlepart(
                                Attachment::new_inline(String::from("125"))
                                    .body(self.logo_ala.clone(), "image/png".parse().unwrap()),
                            ),
                    ),
            )?;
        self.send(message).await?;
        Ok(())
    }

    pub async fn send_change_password_mail(
        &self,
        email: &str,
        user_name: &str,
        link: &str,
    ) -> Result<(), Box<dyn Error>> {
        let plain = self
            .change_password_mail_content_plain
            .replace("{user}", user_name)
            .replace("{email}", email)
            .replace("{link}", link);
        let html: String = self
            .change_password_mail_content
            .replace("{user}", user_name)
            .replace("{email}", email)
            .replace("{link}", link);

        let message = Message::builder()
            .from("Insigno <insigno@mindshub.it>".parse().unwrap())
            .to(email.parse().unwrap())
            .subject("Cambio password account Insigno")
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::plain(plain))
                    .multipart(
                        MultiPart::related()
                            .singlepart(SinglePart::html(html))
                            .singlepart(
                                Attachment::new_inline(String::from("123"))
                                    .body(self.logo_insigno.clone(), "image/png".parse().unwrap()),
                            )
                            .singlepart(
                                Attachment::new_inline(String::from("124"))
                                    .body(self.logo_mindshub.clone(), "image/png".parse().unwrap()),
                            )
                            .singlepart(
                                Attachment::new_inline(String::from("125"))
                                    .body(self.logo_ala.clone(), "image/png".parse().unwrap()),
                            ),
                    ),
            )?;
        self.send(message).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use lettre::{
        transport::smtp::{authentication::Credentials, PoolConfig},
        AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    };
    use rocket::{
        figment::providers::{Format, Toml},
        Config,
    };

    use crate::InsignoConfig;

    use super::MailBuilder;

    #[rocket::async_test]
    async fn test() {
        let figment = Config::figment().merge(Toml::file("Insigno.toml").nested());
        let config: InsignoConfig = figment.extract().unwrap();

        let mail_config = PoolConfig::new().min_idle(1);
        let creds = Credentials::new(
            config.smtp.user.to_string(),
            config.smtp.password.to_string(),
        );
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp.server)
                .unwrap()
                .credentials(creds)
                .pool_config(mail_config)
                .build();

        let z = MailBuilder::new(&config).await.unwrap();
        let mail = z
            .send_registration_mail("test@test.test", "test", "test.com")
            .await;
    }
}
