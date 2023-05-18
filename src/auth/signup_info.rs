use std::mem;

use diesel::{sql_query, sql_types::Text, RunQueryDsl};
use rocket::tokio::task::spawn_blocking;
use serde::Deserialize;

use crate::{db::Db, utils::InsignoError, InsignoConfig};

use super::{
    user::User,
    validation::{Email, Name, Password, SanitizeEmail, SanitizeName, SanitizePassword},
};

#[derive(FromForm, Deserialize, Clone)]
pub struct SignupInfo {
    pub name: String,
    pub email: String,
    pub password: String,
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

impl Email for SignupInfo {
    fn get_email(&mut self) -> &mut String {
        &mut self.email
    }
}

impl SignupInfo {
    pub async fn check(
        &mut self,
        connection: &Db,
        config: &InsignoConfig,
    ) -> Result<(), InsignoError> {
        self.sanitize_name()
            .map_err(|e| InsignoError::new(422, e, e))?;
        self.sanitize_email()
            .map_err(|e| InsignoError::new(422, e, e))?;
        let mut me= self.clone();
        let c = config.clone();
        let mut me = spawn_blocking(move ||->Result<_, _>{
            me.sanitize_password(&c)
                .map_err(|e| InsignoError::new(422, e, e))?;
            Ok(me)
        }
        ).await.unwrap()?;
        mem::swap(self, &mut me);
        //check if unique
        let name = self.name.to_string();
        let email = self.email.to_string();
        let ret: Vec<User> = connection
            .run(move |conn| {
                sql_query("SELECT * FROM users WHERE email=$1 OR name=$2;")
                    .bind::<Text, _>(email)
                    .bind::<Text, _>(name)
                    .get_results(conn)
            })
            .await
            .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
        if !ret.is_empty() {
            let message = "email o nome utente già utilizzati";
            return Err(InsignoError::new(401, message, message));
        }
        Ok(())
    }
}
