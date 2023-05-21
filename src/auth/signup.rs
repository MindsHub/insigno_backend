use rocket::{form::Form, tokio::task::spawn_blocking, State, http::ContentType};
use scrypt::Params;
use serde::Deserialize;

use crate::{db::Db, utils::InsignoError, InsignoConfig, mail::MailBuilder, pending::{Pending, PendingAction}};

use super::{
    user::{Unauthenticated, User},
    validation::{Email, Name, Password, SanitizeEmail, SanitizeName, SanitizePassword}
};

#[derive(FromForm, Deserialize, Clone)]
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
            .map_err(|x| InsignoError::new(401, x, x))?;
        self.sanitize_name()
            .map_err(|x| InsignoError::new(401, x, x))?;
        self.sanitize_password()
            .map_err(|x| InsignoError::new(401, x, x))?;
        Ok(())
    }
}
/*
impl User<Unauthenticated> {
    async fn from(
        String string, string
    ) -> Result<Self, InsignoError> {
        let value = spawn_blocking(move || {
            value
                .hash_password(&params.into())
                .map_err(|e| InsignoError::new_debug(501, &e.to_string()))
                .map(|_| value)
        })
        .await
        .map_err(|e| InsignoError::new_debug(501, &e.to_string()))??;

        Ok(Self {
            id: None,
            name: value.name,
            email: value.email,
            password_hash: value.password,
            is_admin: false,
            points: 0.0,
            phantom: std::marker::PhantomData,
        })
    }
}*/

#[post("/signup", format = "form", data = "<create_info>")]
pub async fn signup(
    mut create_info: Form<SignupInfo>,
    mailer: &State<MailBuilder>,
    config: &State<InsignoConfig>,
    connection: Db,
) -> Result<String, InsignoError> {
    create_info.sanitize()?;
    let params: Params = config.scrypt.clone().into();
    let create_info = spawn_blocking(move || {
        create_info
            .hash_password(&params)
            .map_err(|e| InsignoError::new_debug(501, &e.to_string()))
            .map(|_| create_info)
    })
    .await
    .map_err(|e| InsignoError::new_debug(501, &e.to_string()))??;

    //create_info.hash_password(config.scrypt)?;
    let mut pend = Pending::new(PendingAction::RegisterUser(create_info.name.clone(), create_info.email.clone(), create_info.password.clone()));
    pend.insert(&connection).await?;
    let link = format!("https://insigno.mindshub.it/verify/{}", pend.token);
    mailer.send_registration_mail(&create_info.email, &create_info.name, &link).await.map_err(|e| InsignoError::new_debug(501, &e.to_string()))?;

    Ok("mail inviata".to_string())
}

pub async fn complete_registration(pend: PendingAction, connection: &Db)->Result<(ContentType, String), InsignoError>{
    if let PendingAction::RegisterUser(name, email, password_hash) = pend{
        let mut user = User{
            id: None,
            name,
            email,
            password_hash,
            is_admin: false,
            points: 0.0,
            phantom: std::marker::PhantomData::<Unauthenticated>,
        };
        user.insert(connection).await?;
        println!("registrazione completata");
        Ok((ContentType::HTML, "registrazione completata".to_string()))
    }else{
        Err(InsignoError::new_debug(500, "wrong call"))
    }
}