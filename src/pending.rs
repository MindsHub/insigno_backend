use std::mem;

use chrono::Utc;
use diesel::{insert_into, sql_query, sql_types::Text};

use rocket::{fairing::AdHoc, http::ContentType, serde::json::serde_json, State};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{
        change_password::complete_change_password, delete_account::complete_delete,
        signup::complete_registration,
    },
    db::Db,
    utils::InsignoError,
    InsignoConfig,
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
    /// user_id, new_hash
    ChangePassword(i64, String),
    ///user_id
    DeleteUser(i64),
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
#[derive(Queryable, Insertable, QueryableByName, Clone, Debug)]
#[diesel(table_name = pending)]
pub struct Pending {
    pub id: Option<i64>,
    pub token: String,
    #[diesel(deserialize_as = String, serialize_as = String)]
    pub action: PendingAction,
    pub request_date: Option<chrono::DateTime<Utc>>,
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
            .map_err(|e| {
                InsignoError::new(422)
                    .client("impossibile creare l'account")
                    .debug(e)
            })?;
        mem::swap(&mut me, self);
        Ok(())
    }

    pub async fn get_from_token(token: &str, connection: &Db) -> Result<Self, InsignoError> {
        if !token.chars().all(|x| x.is_ascii_alphanumeric()) {
            return Err(InsignoError::new(422).both("token_invalido"));
        }
        let token = token.to_string();
        let pending: Self = connection
            .run(|conn| {
                sql_query("SELECT * FROM get_pending($1);")
                    .bind::<Text, _>(token)
                    .get_result(conn)
            })
            .await
            .map_err(|e| InsignoError::new(422).client("token non trovato").debug(e))?;
        Ok(pending)
    }
}

#[get("/verify/<token>")]
pub async fn verify(
    token: &str,
    connection: Db,
    config: &State<InsignoConfig>,
) -> Result<(ContentType, String), InsignoError> {
    let pending = Pending::get_from_token(token, &connection).await?;

    match pending.action {
        PendingAction::RegisterUser(name, email, password) => {
            complete_registration(
                PendingAction::RegisterUser(name, email, password),
                &connection,
            )
            .await
        }
        PendingAction::ChangePassword(user_id, password_hash) => {
            complete_change_password(
                PendingAction::ChangePassword(user_id, password_hash),
                &connection,
            )
            .await
        }
        PendingAction::DeleteUser(user_id) => {
            complete_delete(PendingAction::DeleteUser(user_id), &connection, config).await
        }
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Pending stage", |rocket| async {
        rocket.mount("/", routes![verify])
    })
}
