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
use serde::Deserialize;

use crate::auth::signup_info::SignupInfo;
use crate::auth::user::User;
use crate::utils::InsignoError;
use crate::InsignoConfig;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SmtpConfig {
    server: String,
    user: String,
    password: String,
}

/*#[cfg(test)]
pub async fn send_mail(
    _to: &str,
    _subject: &str,
    _message: &str,
    _mailer: &Mailer,
) -> Result<(), InsignoError> {
    Ok(())
}*/

//#[cfg(not(test))]
pub async fn send_mail(
    to: &str,
    subject: &str,
    message: &str,
    plain_text: &str,
    mailer: &Mailer,
) -> Result<(), InsignoError> {
    let logo_insigno = fs::read("./templates/logo_insigno.png")
        .await
        .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
    let logo_insigno_body = Body::new(logo_insigno);

    let logo_ala = fs::read("./templates/logo_ala.png")
        .await
        .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
    let logo_ala_body = Body::new(logo_ala);

    let logo_mindshub = fs::read("./templates/logo_mindshub.png")
        .await
        .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
    let logo_mindshub_body = Body::new(logo_mindshub);

    //let html = String::from_utf8(fs::read("./templates/insigno.html").unwrap()).unwrap();
    let m = Message::builder()
        .from("Insigno <insigno@mindshub.it>".parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(String::from(plain_text)))
                .multipart(
                    MultiPart::related()
                        .singlepart(SinglePart::html(String::from(
                            message,
                        )))
                        .singlepart(
                            Attachment::new_inline(String::from("123"))
                                .body(logo_insigno_body, "image/png".parse().unwrap()),
                        )
                        .singlepart(
                            Attachment::new_inline(String::from("124"))
                                .body(logo_mindshub_body, "image/png".parse().unwrap()),
                        )
                        .singlepart(
                            Attachment::new_inline(String::from("125"))
                                .body(logo_ala_body, "image/png".parse().unwrap()),
                        ),
                ),
        )
        .unwrap();
    match mailer.m.send(m).await {
        Ok(resp) => {

            if resp.is_positive() {
                Ok(())
            } else {
                Err(InsignoError::new_debug(500, resp.first_line().unwrap()))
            }
        }
        Err(e) => Err(InsignoError::new_debug(500, &e.to_string())),
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
