use std::{mem};

use chrono::Utc;
use diesel::{insert_into, sql_types::Text, sql_query};

use rocket::{serde::json::serde_json, http::ContentType, fairing::AdHoc};
use serde::{Deserialize, Serialize};

use crate::{
    db::Db,
    utils::InsignoError, auth::signup::complete_registration,
};
use diesel::RunQueryDsl;

#[cfg(not(test))]
pub fn generate_token() -> String {
    use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
    OsRng
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect()
}

#[cfg(test)]
pub fn generate_token() -> String {
    "11111111111111111111".to_string()
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum PendingAction {
    /// name, email, password
    RegisterUser(String, String, String),
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
        token->Text,
        action->Text,
        request_date->Nullable<Timestamptz>,
    }
}
#[derive(Queryable, Clone, Insertable, QueryableByName, Debug)]
#[diesel(table_name = pending)]
pub struct Pending {
    id: Option<i64>,
    pub token: String,
    #[diesel(deserialize_as = String, serialize_as = String)]
    pub action: PendingAction,
    pub request_date: Option<chrono::DateTime<Utc>>
}

impl Pending {
    pub fn new(action: PendingAction) -> Self {
        let token = generate_token();

        Pending {
            id: None,
            token,
            action,
            request_date: None,
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
        println!("inserted {:?}", self);
        Ok(())
    }

    pub async fn get_from_token(token: String, connection: &Db)->Result<Self, InsignoError>{
        if !token.chars().all(|x| x.is_ascii_alphanumeric()) {
            let s = "token invalido";
            return Err(InsignoError::new(422, s, s));
        }
        let token = token.to_string();
        println!("token={}", token);
        let pending: Self = connection
            .run(|conn| {
                sql_query("SELECT * FROM get_pending($1);")
                    .bind::<Text, _>(token)
                    .get_result(conn)
            })
            .await
            .map_err(|e| InsignoError::new(422, "token non trovato", &e.to_string()))?;
        Ok(pending)
    }
}

#[get("/verify/<token>")]
pub async fn verify(
    token: String,
    connection: Db,
) -> Result<(ContentType, String), InsignoError> {
    println!("verificando");
    let pending = Pending::get_from_token(token, &connection).await?;
    
    match pending.action{
        PendingAction::RegisterUser(name, email, password) => {complete_registration(PendingAction::RegisterUser(name, email, password), &connection).await},
        PendingAction::ChangePassword(_, _) => {todo!()},
    }

    /*let success = fs::read("./templates/account_creation.html")
        .await
        .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;

    let success =
        String::from_utf8(success).map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;*/

    
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Pending stage", |rocket| async {
        rocket.mount("/", routes![verify])
    })
}

#[cfg(test)]
mod test {
    use crate::pending::PendingAction;


    #[rocket::async_test]
    async fn test() {
        println!(
            "{:?}",
            String::try_from(PendingAction::ChangePassword(15, "test".to_string()))
        );
    }
}
