use crypto::scrypt::{self, ScryptParams};
use diesel::{sql_query, sql_types::*};
use rocket::{
    http::Status,
    request::{self, FromRequest},
};
use serde::Deserialize;

use crate::{
    db::Db,
    schema_rs::{PendingUser, User},
    schema_sql::*,
    utils::InsignoError,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub fn hash_password(password: &String) -> String {
    let params = ScryptParams::new(11, 8, 1);
    scrypt::scrypt_simple(password, &params).unwrap()
}

pub fn check_hash(password: &String, hash: &String) -> bool {
    scrypt::scrypt_check(password, hash).unwrap()
}

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

pub async fn get_user_by_email(db: &Db, email: String) -> Result<User, diesel::result::Error> {
    let users: Vec<User> = db
        .run(|conn| {
            users::table
                .filter(users::email.eq(email))
                .get_results(conn)
        })
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

#[derive(FromForm, Deserialize, Clone)]
pub(crate) struct SignupInfo {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(FromForm, Deserialize)]
pub(crate) struct LoginInfo {
    pub email: String,
    pub password: String,
}

impl From<SignupInfo> for LoginInfo {
    fn from(value: SignupInfo) -> Self {
        Self {
            email: value.email,
            password: value.password,
        }
    }
}

impl SignupInfo {
    pub async fn check(&mut self, db: &Db) -> Result<(), InsignoError> {
        let mut check = || -> Result<(), String> {
            self.check_name()?;
            self.check_email()?;
            self.check_password()?;
            Ok(())
        };
        check().map_err(|e| InsignoError::new(401, &e, &e))?;
        self.check_unique(db).await?;
        Ok(())
    }

    async fn check_unique(&self, db: &Db) -> Result<(), InsignoError> {
        let email = self.email.to_string();
        let name = self.name.to_string();
        let ret: Vec<User> = db
            .run(move |conn| {
                sql_query("SELECT * FROM users WHERE email=$1 OR name=$2;")
                    .bind::<Text, _>(email)
                    .bind::<Text, _>(name)
                    .get_results(conn)
            })
            .await
            .map_err(|x| InsignoError::new_debug(500, &x.to_string()))?;
        if ret.len() > 0 {
            let message = "email o nome utente già utilizzati";
            Err(InsignoError::new(401, message, message))
        } else {
            Ok(())
        }
    }

    fn check_name(&mut self) -> Result<(), &str> {
        self.name = self.name.trim().to_string();
        let name_len = self.name.len();
        if name_len < 3 && 20 < name_len {
            return Err("Nome utente invalido. Deve essere lungo tra 3 e 20 caratteri (e possibilmente simile al nome)");
        }
        if !self
            .name
            .chars()
            .all(|x| x.is_alphanumeric() || x == '_' || x == ' ')
        {
            return Err(
                "Nome utente invalido. Un nome corretto può contenere lettere, numeri, spazi e _",
            );
        }
        Ok(())
    }

    fn check_password(&self) -> Result<(), &str> {
        if self.password.len() < 8 {
            return Err("Password troppo breve, deve essere lunga almeno 8 caratteri");
        }

        if !self.password.chars().any(|x| x.is_ascii_uppercase()) {
            return Err("La password deve contenere almeno una maiuscola");
        }

        if !self.password.chars().any(|x| x.is_ascii_lowercase()) {
            return Err("La password deve contenere almeno una minuscola");
        }

        if !self.password.chars().any(|x| x.is_numeric()) {
            return Err("La password deve contenere almeno un numero");
        }

        if !self.password.chars().any(|x| !x.is_ascii_alphanumeric()) {
            return Err("La password deve contenere almeno un carattere speciale");
        }

        Ok(())
    }

    fn check_email(&self) -> Result<(), &str> {
        if !self
            .email
            .chars()
            .all(|x| x.is_ascii_alphanumeric() || x == '.' || x == '-' || x == '@' || x == '_')
        {
            return Err("Mail invalida");
        }

        Ok(())
    }
}

impl From<SignupInfo> for PendingUser {
    fn from(value: SignupInfo) -> Self {
        PendingUser {
            id: None,
            name: value.name,
            email: value.email,
            password_hash: hash_password(&value.password),
            request_date: None,
            token: generate_token(),
        }
    }
}
