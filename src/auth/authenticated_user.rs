use diesel::{
    sql_query,
    sql_types::{BigInt, Text},
    RunQueryDsl,
};
use rocket::request::{self, FromRequest};
use serde::{ser::SerializeStruct, Serialize};

use crate::{db::Db, utils::InsignoError};

use super::user::User;

pub struct AuthenticatedUser {
    user: User,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = InsignoError;

    async fn from_request(request: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
        let connection = request.guard::<Db>().await.unwrap();
        let cookie = request.cookies();
        let insigno_auth = match cookie.get_private("insigno_auth") {
            Some(a) => a,
            None => {
                return InsignoError::new_debug(401, "insigno_auth cookie not found").into();
            }
        }
        .value()
        .to_string();
        let vec: Vec<&str> = insigno_auth.split(' ').collect();

        let id: i64 = vec[0].parse().unwrap();
        let tok = vec[1].to_string();
        if !tok.chars().all(|x| x.is_ascii_alphanumeric()) {
            return InsignoError::new_debug(401, "sql injection?").into();
        }

        let auth: Result<User, _> = connection
            .run(move |conn| {
                sql_query("SELECT * FROM autenticate($1, $2);")
                    .bind::<BigInt, _>(id)
                    .bind::<Text, _>(tok)
                    .get_result(conn)
            })
            .await;

        match auth {
            Ok(a) => {
                let auth = AuthenticatedUser { user: a };
                return request::Outcome::Success(auth);
            }
            Err(e) => {
                return InsignoError::new(401, "Authentication error", &e.to_string()).into();
            }
        }
    }
}
impl AsRef<User> for AuthenticatedUser {
    fn as_ref(&self) -> &User {
        &self.user
    }
}
impl Serialize for AuthenticatedUser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("User", 3)?;
        s.serialize_field("id", &self.user.id)?;
        s.serialize_field("name", &self.user.name)?;
        s.serialize_field("points", &self.user.points)?;
        s.end()
    }
}
