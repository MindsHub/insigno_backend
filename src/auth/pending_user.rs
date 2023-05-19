use std::{fs, mem};

use chrono::Utc;
use diesel::{insert_into, sql_query, sql_types::*};
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};

use crate::{
    db::Db,
    mail::{send_mail, Mailer},
    utils::InsignoError,
    InsignoConfig,
};
use diesel::RunQueryDsl;

use super::signup::SignupInfo;

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
    pub async fn new(
        mut value: SignupInfo,
        connection: &Db,
        config: &InsignoConfig,
    ) -> Result<Self, InsignoError> {
        //it hash the password
        //value.check(connection, config).await?;
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

        let mail = fs::read("./templates/mail_account_creation.html") // TODO cache file
            .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
        let mail =
            String::from_utf8(mail).map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
        let mail = mail
            .replace("{user}", &self.name)
            .replace("{email}", &self.email)
            .replace("{link}", &link);
        let alternative_mail = fs::read("./templates/mail_account_creation_plain.txt")
            .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
        let alternative_mail = String::from_utf8(alternative_mail)
            .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
        let alternative_mail = alternative_mail
            .replace("{user}", &self.name)
            .replace("{email}", &self.email)
            .replace("{link}", &link);
        send_mail(
            &self.email,
            "Verifica account",
            &mail,
            &alternative_mail,
            mailer,
        )
        .await
    }

    pub async fn register_and_mail(
        self,
        connection: &Db,
        mailer: &Mailer,
    ) -> Result<(), InsignoError> {
        self.send_verification_mail(mailer).await?;
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

#[derive(Deserialize, Serialize, Clone)]
pub enum PendingAction {
    RegisterUser(i64),
    ChangePassword(i64, String),
}
impl From<String> for PendingAction {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}
impl From<PendingAction> for String {
    fn from(value: PendingAction) -> Self {
        serde_json::to_string(&value).unwrap()
    }
}

table! {
    pending(id){
        id->Nullable<BigInt>,
        key->Text,
        action->Text,
    }
}
#[derive(Queryable, Clone, Insertable)]
#[diesel(table_name = pending)]
pub struct Pending {
    id: Option<i64>,
    key: String,
    #[diesel(deserialize_as = String, serialize_as = String)]
    action: PendingAction,
}

impl Pending {
    pub fn new(action: PendingAction) -> Self {
        let key = OsRng
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        Pending {
            id: None,
            key,
            action,
        }
    }
    pub async fn insert(&mut self, connection: &Db) -> Result<(), InsignoError> {
        let me: Self = self.clone();
        let mut me: Self = connection
            .run(|conn| {
                insert_into(pending::dsl::pending)
                    .values(me)
                    .get_result(conn)
            })
            .await
            .map_err(|e| InsignoError::new(422, "impossibile creare l'account", &e.to_string()))?;
        mem::swap(&mut me, self);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::auth::pending_user::PendingAction;

    #[rocket::async_test]
    async fn test() {
        println!(
            "{:?}",
            String::try_from(PendingAction::ChangePassword(15, "test".to_string()))
        );
    }
}
