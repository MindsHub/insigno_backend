use diesel::{sql_query, sql_types::Text, RunQueryDsl};
use serde::Deserialize;

use crate::{db::Db, utils::InsignoError};

use super::{validation::{Name, Password, Email, SanitizeName, SanitizeEmail, SanitizePassword}, user::User};

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
    pub async fn check(&mut self, connection: &Db) -> Result<(), InsignoError> {
        self.sanitize_name()
            .map_err(|e| InsignoError::new(422, e, e))?;
        self.sanitize_email()
            .map_err(|e| InsignoError::new(422, e, e))?;
        self.sanitize_password()
            .map_err(|e| InsignoError::new(422, e, e))?;

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