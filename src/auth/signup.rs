use std::mem;

use diesel::{sql_query, sql_types::Text, RunQueryDsl};
use rocket::{form::Form, tokio::task::spawn_blocking, State};
use scrypt::Params;
use serde::Deserialize;

use crate::{db::Db, mail::Mailer, utils::InsignoError, InsignoConfig};

use super::{
    scrypt::{scrypt_simple, InsignoScryptParams},
    user::{Unauthenticated, User},
    validation::{Email, Name, Password, SanitizeEmail, SanitizeName, SanitizePassword},
    Pending, PendingAction,
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

impl User<Unauthenticated> {
    async fn from(
        mut value: SignupInfo,
        params: InsignoScryptParams,
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
}

#[post("/signup", format = "form", data = "<create_info>")]
pub async fn signup(
    mut create_info: Form<SignupInfo>,
    mail_cfg: &State<Mailer>,
    config: &State<InsignoConfig>,
    connection: Db,
) -> Result<String, InsignoError> {
    create_info.sanitize()?;

    let mut user = User::from(create_info.into_inner(), config.scrypt.clone().into()).await?;
    user.insert(&connection).await?;
    let mut pend = Pending::new(PendingAction::RegisterUser(user.id.unwrap()));
    pend.insert(&connection).await?;
    //send registration mail and insert it in db
    //pending.register_and_mail(&db, mail_cfg).await?;

    Ok("mail inviata".to_string())
}
