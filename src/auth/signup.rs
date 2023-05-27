use std::mem;

use rocket::{form::Form, http::ContentType, tokio::task::spawn_blocking, State};

use crate::{
    db::Db,
    mail::MailBuilder,
    pending::{Pending, PendingAction},
    utils::InsignoError,
    InsignoConfig,
};

use super::{
    user::{Unauthenticated, User},
    validation::{Email, Name, Password, SanitizeEmail, SanitizeName, SanitizePassword},
};

#[derive(FromForm)]
pub struct SignupInfo {
    pub name: String,
    pub email: String,
    pub password: String,
}
// validation on this struct
impl Email for SignupInfo {
    fn get_email(&mut self) -> &mut String {
        &mut self.email
    }
}
impl Name for SignupInfo {
    fn get_name(&mut self) -> &mut String {
        &mut self.name
    }
}
impl Password for SignupInfo {
    fn get_password(&mut self) -> &mut String {
        &mut self.password
    }
}

impl SignupInfo {
    pub fn sanitize(&mut self) -> Result<(), InsignoError> {
        self.sanitize_email()
            .map_err(|x| InsignoError::new(401).both(x))?;
        self.sanitize_name()
            .map_err(|x| InsignoError::new(401).both(x))?;
        self.sanitize_password()
            .map_err(|x| InsignoError::new(401).both(x))?;
        Ok(())
    }
}

#[post("/signup", format = "form", data = "<create_info>")]
pub async fn signup(
    mut create_info: Form<SignupInfo>,
    mailer: &State<MailBuilder>,
    config: &State<InsignoConfig>,
    connection: Db,
) -> Result<String, InsignoError> {
    create_info.sanitize()?;
    let permit = config.scrypt.clone().await;
    let params = permit.get_params();
    let create_info = spawn_blocking(move || {
        create_info
            .hash_password(&params)
            .map_err(|e| InsignoError::new(501).debug(e))
            .map(|_| create_info)
    })
    .await
    .map_err(|e| InsignoError::new(501).debug(e))??;
    mem::drop(permit);

    let mut pend = Pending::new(PendingAction::RegisterUser(
        create_info.name.clone(),
        create_info.email.clone(),
        create_info.password.clone(),
    ));
    pend.insert(&connection).await?;
    let link = format!("https://insigno.mindshub.it/verify/{}", pend.token);
    mailer
        .send_registration_mail(&create_info.email, &create_info.name, &link)
        .await
        .map_err(|e| InsignoError::new(501).debug(e))?;

    Ok("mail inviata".to_string())
}

pub async fn complete_registration(
    pend: PendingAction,
    connection: &Db,
) -> Result<(ContentType, String), InsignoError> {
    if let PendingAction::RegisterUser(name, email, password_hash) = pend {
        let mut user = User {
            id: None,
            name,
            email,
            password_hash,
            is_admin: false,
            points: 0.0,
            phantom: std::marker::PhantomData::<Unauthenticated>,
        };
        user.insert(connection).await?;
        Ok((ContentType::HTML, "registrazione completata".to_string()))
    } else {
        Err(InsignoError::new(500).debug("wrong call"))
    }
}

#[cfg(test)]
mod test {
    use crate::rocket;
    use rocket::{http::ContentType, local::asynchronous::Client};
    #[rocket::async_test]
    async fn test_empty_string() {
        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");

        let _response = client
            .post("/signup")
            .header(ContentType::Form)
            .body("")
            .dispatch()
            .await;
    }
}
