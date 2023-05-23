use rocket::{form::Form, http::ContentType, tokio::task::spawn_blocking, State};

use crate::{
    db::Db,
    mail::MailBuilder,
    pending::{Pending, PendingAction},
    utils::InsignoError,
    InsignoConfig,
};

use super::{
    user::User,
    validation::{Email, Password, SanitizeEmail, SanitizePassword},
};

#[derive(FromForm)]
pub struct ChangePasswordRequest {
    email: String,
    password: String,
}
impl Email for ChangePasswordRequest {
    fn get_email(&mut self) -> &mut String {
        &mut self.email
    }
}

impl Password for ChangePasswordRequest {
    fn get_password(&mut self) -> &mut String {
        &mut self.password
    }
}
impl ChangePasswordRequest {
    fn sanitize(&mut self) -> Result<(), InsignoError> {
        self.sanitize_email()
            .map_err(|x| InsignoError::new(401, x, x))?;
        self.sanitize_password()
            .map_err(|x| InsignoError::new(401, x, x))?;
        Ok(())
    }
}

#[post(
    "/change_password",
    format = "form",
    data = "<change_password_request>"
)]
pub async fn change_password(
    db: Db,
    mut change_password_request: Form<ChangePasswordRequest>,
    mailer: &State<MailBuilder>,
    config: &State<InsignoConfig>,
) -> Result<String, InsignoError> {
    change_password_request.sanitize()?;
    let params = config.scrypt.clone().into();
    let change_password_request = spawn_blocking(move || {
        change_password_request
            .hash_password(&params)
            .map_err(|e| InsignoError::new_debug(501, &e.to_string()))
            .map(|_| change_password_request)
    })
    .await
    .map_err(|e| InsignoError::new_debug(501, &e.to_string()))??;
    let _: Result<(), InsignoError> = async move {
        let user = User::get_by_email(&db, change_password_request.email.clone()).await?;
        let mut pending: Pending = Pending::new(PendingAction::ChangePassword(
            user.get_id(),
            change_password_request.password.clone(),
        ));
        pending.insert(&db).await?;
        let link = format!("https://insigno.mindshub.it/verify/{}", pending.token);
        mailer
            .send_change_password_mail(&user.email, &user.name, &link)
            .await
            .map_err(|_| InsignoError::new_code(500))?;
        Ok(())
    }
    .await;

    Ok("Abbiamo inviato una mail all'utente interessato".to_string())
}

pub async fn complete_change_password(
    pend: PendingAction,
    connection: &Db,
) -> Result<(ContentType, String), InsignoError> {
    if let PendingAction::ChangePassword(user_id, password_hash) = pend {
        let mut user = User::get_by_id(connection, user_id).await?;
        user.password_hash = password_hash;
        user.update(connection).await?;
        Ok((
            ContentType::HTML,
            "Password cambiata con successo".to_string(),
        ))
    } else {
        Err(InsignoError::new_debug(500, "wrong call"))
    }
}
