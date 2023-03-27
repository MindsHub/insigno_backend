use std::fs;

use crate::{
    mail::{send_mail, SmtpConfig},
    schema_rs::PendingUser,
    utils::InsignoError,
};

impl PendingUser {
    pub(crate) fn send_verification_mail(&self, config: &SmtpConfig) -> Result<(), InsignoError> {
        let link = format!("https://insigno.mindshub.it/verify/{}", self.token);

        let mail = fs::read("./templates/mail.html")
            .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
        let mail =
            String::from_utf8(mail).map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
        let mail = mail
            .replace("{user}", &self.name)
            .replace("{mail}", &self.email)
            .replace("{link}", &link);
        send_mail(&self.email, "Verifica account", &mail, config)
    }
}
