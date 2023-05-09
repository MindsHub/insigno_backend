use crate::diesel::query_dsl::methods::FilterDsl;
use crate::diesel::ExpressionMethods;
use crate::{db::Db, utils::InsignoError};
use super::scrypt;
use diesel::insert_into;
use diesel::RunQueryDsl;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

use super::login_info::LoginInfo;
use super::validation::{SanitizeEmail, SanitizePassword};
use super::PendingUser;
table! {
    users(id){
        id -> Nullable<BigInt>,
        name -> Text,
        email -> Text,
        password -> Text,
        is_admin -> Bool,
        points -> Double,
    }
}

#[derive(Debug, Clone, Default, QueryId, Deserialize, Insertable, Queryable, QueryableByName)]
#[diesel(table_name = users)]
/** generic user interface for db (not authenticated)*/
pub struct User {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub is_admin: bool,
    pub points: f64,
}

impl User {
    pub async fn get_by_email(db: &Db, email: String) -> Result<Self, InsignoError> {
        let user: Self = db
            .run(|conn| users::table.filter(users::email.eq(email)).get_result(conn))
            .await
            .map_err(|e| InsignoError::new_debug(404, &e.to_string()))?;
        Ok(user)
    }
    pub async fn get_by_id(db: &Db, id_user: i64) -> Result<Self, InsignoError> {
        let user: Self = db
            .run(move |conn| users::table.filter(users::id.eq(id_user)).get_result(conn))
            .await
            .map_err(|e| InsignoError::new_debug(404, &e.to_string()))?;
        Ok(user)
    }
    pub async fn new_from_pending(v: PendingUser, connection: &Db) -> Result<Self, InsignoError> {
        let user = User {
            id: None,
            name: v.name,
            email: v.email,
            password: v.password_hash,
            is_admin: false,
            points: 0.0,
        };

        let user: User = connection
            .run(|conn| insert_into(users::dsl::users).values(user).get_result(conn))
            .await
            .map_err(|e| InsignoError::new(422, "impossibile creare l'account", &e.to_string()))?;

        Ok(user)
    }
    pub async fn login(mut v: LoginInfo, connection: &Db) -> Result<Self, InsignoError> {
        v.fmt_password();
        v.fmt_email();

        let user = User::get_by_email(connection, v.email)
            .await
            .map_err(|_| InsignoError::new(401, "invalid user", "invalid user"))?;
        if !user.check_hash(&v.password) {
            let message = "email o password errati";
            Err(InsignoError::new(403, message, message))
        } else {
            Ok(user)
        }
    }
    pub fn check_hash(&self, password: &str) -> bool {
        scrypt::scrypt_check(password, &self.password).unwrap()
    }
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("User", 3)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("points", &self.points)?;
        s.end()
    }
}
