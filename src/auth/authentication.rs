use crypto::scrypt::{ScryptParams, self};
use diesel::{sql_query, sql_types::*};
use rand::{distributions::Alphanumeric, Rng};
use rocket::{request::{self, FromRequest}, http::Status};

use crate::{schema_rs::User, db::Db, schema_sql::*};
use diesel::{QueryDsl, ExpressionMethods, RunQueryDsl};



pub fn hash_password(password: &String) -> String {
    let params = ScryptParams::new(11, 8, 1);
    scrypt::scrypt_simple(password, &params).unwrap()
}

pub fn check_hash(password: &String, hash: &String) -> bool{
    scrypt::scrypt_check(password, hash).unwrap()
}

pub fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}

pub async fn get_user_by_email(db: &Db, email: String) -> Result<User, diesel::result::Error> {
    let users: Vec<User> = db
        .run(|conn| users::table.filter(users::email.eq(email)).get_results(conn))
        .await?;
    //.map_err(to_debug)?;
    let user = users.get(0).ok_or(diesel::result::Error::NotFound)?;
    Ok(user.clone())
}

pub async fn get_user_by_id(db: &Db, id_user: i64) -> Result<User, diesel::result::Error> {
    let users: Vec<User> = db
        .run(move |conn| users::table.filter(users::id.eq(id_user)).get_results(conn))
        .await?;
    //.map_err(to_debug)?;
    let user = users.get(0).ok_or(diesel::result::Error::NotFound)?;
    Ok(user.clone())
}

#[derive(Responder, Debug)]
pub enum AuthError<T> {
    #[response(status = 401)]
    Unauthorized(T),
}
fn auth_fail(inp: &str) -> request::Outcome<User, AuthError<String>> {
    request::Outcome::Failure((
        Status::Unauthorized,
        AuthError::Unauthorized(inp.to_string()),
    ))
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = AuthError<String>;

    async fn from_request(request: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
        let connection = request.guard::<Db>().await.unwrap();
        let cookie = request.cookies();
        let insigno_auth = match cookie.get_private("insigno_auth") {
            Some(a) => a,
            None => {
                return auth_fail("insigno_auth cookie not found");
            }
        }
        .value()
        .to_string();
        let vec: Vec<&str> = insigno_auth.split(' ').collect();

        let id: i64 = vec[0].parse().unwrap();
        let tok = vec[1].to_string();
        if !tok.chars().all(|x| x.is_ascii_alphanumeric()) {
            return auth_fail("sql ignection?");
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
                return request::Outcome::Success(a);
            }
            Err(_) => {
                return auth_fail("errore nell'autenticazione");
            }
        }
    }
}