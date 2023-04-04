use std::fs;

use chrono::Utc;
use diesel::{sql_query, sql_types::*};

use crate::{
    db::Db,
    mail::{send_mail, Mailer},
    utils::InsignoError,
};
use diesel::RunQueryDsl;
use serde::Deserialize;

use super::user::User;
use super::validation::{Email, Name, Password, SanitizeEmail, SanitizeName, SanitizePassword};

#[cfg(not(test))]
pub fn generate_token() -> String {
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}

#[cfg(test)]
pub fn generate_token() -> String {
    "1111111111".to_string()
}

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
    async fn check(&mut self, connection: &Db) -> Result<(), InsignoError> {
        self.sanitize_name()
            .map_err(|e| InsignoError::new(422, &e.to_string(), &e.to_string()))?;
        self.sanitize_email()
            .map_err(|e| InsignoError::new(422, &e.to_string(), &e.to_string()))?;
        self.sanitize_password()
            .map_err(|e| InsignoError::new(422, &e.to_string(), &e.to_string()))?;

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

/*

impl LoginInfo {
    pub async fn check(&mut self) -> Result<(), InsignoError> {
        let mut check = || -> Result<(), String> {
            self.email = check_email(&self.email)?;
            self.password = check_password(&self.password)?;
            Ok(())
        };
        check().map_err(|e| InsignoError::new(401, &e, &e))?;
        Ok(())
    }
}*/

table! {
    pending_users(id){
        id->Nullable<BigInt>,
        name->Text,
        email->Text,
        password_hash->Text,
        request_date->Nullable<Timestamptz>,
        token->Text,
    }
}

#[derive(Queryable, Insertable, Debug, QueryableByName)]
#[diesel(table_name = pending_users)]
pub struct PendingUser {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub request_date: Option<chrono::DateTime<Utc>>,
    pub token: String,
}

impl PendingUser {
    pub async fn new(mut value: SignupInfo, connection: &Db) -> Result<Self, InsignoError> {
        //it hash the password
        value.check(connection).await?;
        Ok(PendingUser {
            id: None,
            name: value.name,
            email: value.email,
            password_hash: value.password,
            request_date: None,
            token: generate_token(),
        })
    }

    pub async fn send_verification_mail(&self, mailer: &Mailer) -> Result<(), InsignoError> {
        let link = format!("https://insigno.mindshub.it/verify/{}", self.token);

        let mail = fs::read("./templates/mail.html") // TODO cache file
            .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
        let mail =
            String::from_utf8(mail).map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
        let mail = mail
            .replace("{user}", &self.name)
            .replace("{mail}", &self.email)
            .replace("{link}", &link);
        send_mail(&self.email, "Verifica account", &mail, mailer).await
    }

    pub async fn register_and_mail(
        self,
        connection: &Db,
        mailer: &Mailer,
    ) -> Result<(), InsignoError> {
        self.send_verification_mail(mailer).await;
        connection
            .run(|conn| {
                use pending_users::dsl::pending_users;
                diesel::insert_into(pending_users)
                    .values(self)
                    .execute(conn)
            }) //::table.filter(users::email.eq(email)).get_result(conn))
            .await
            .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
        Ok(())
    }

    pub async fn from_token(token: String, connection: &Db) -> Result<Self, InsignoError> {
        if !token.chars().all(|x| x.is_ascii_alphanumeric()) {
            let s = "token invalido";
            return Err(InsignoError::new(422, s, s));
        }
        let pending_user: PendingUser = connection
            .run(|conn| {
                sql_query("SELECT * FROM get_pending_user($1);")
                    .bind::<Text, _>(token)
                    .get_result(conn)
            })
            .await
            .map_err(|e| InsignoError::new(422, "token non trovato", &e.to_string()))?;
        Ok(pending_user)
    }
}
