use std::marker::PhantomData;
use std::mem;

use super::scrypt;
use crate::diesel::query_dsl::methods::FilterDsl;
use crate::diesel::ExpressionMethods;
use crate::{db::Db, utils::InsignoError};
use diesel::sql_types::{BigInt, Text};
use diesel::RunQueryDsl;
use diesel::{insert_into, sql_query};
use rocket::request::{self, FromRequest, Outcome};
use rocket::tokio::task::spawn_blocking;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

use super::login_info::LoginInfo;
use super::validation::{SanitizeEmail, SanitizePassword};

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

#[derive(Clone)]
pub enum Unauthenticated {}

#[derive(Clone)]
pub enum Authenticated {}
#[derive(Clone)]
pub enum AuthenticatedAdmin {}

pub trait UserType: Clone + Send {}
impl UserType for Unauthenticated {}
impl UserType for Authenticated {}
impl UserType for AuthenticatedAdmin {}

#[derive(Debug, Clone, Default, QueryId, Deserialize, Insertable, Queryable, QueryableByName)]
#[diesel(table_name = users)]
struct UserDiesel {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub is_admin: bool,
    pub points: f64,
}

impl<T: UserType> From<UserDiesel> for User<T> {
    fn from(value: UserDiesel) -> User<T> {
        Self {
            id: value.id,
            name: value.name,
            email: value.email,
            password_hash: value.password,
            is_admin: value.is_admin,
            points: value.points,
            phantom: PhantomData,
        }
    }
}
impl<T: UserType> From<User<T>> for UserDiesel {
    fn from(value: User<T>) -> Self {
        Self {
            id: value.id,
            name: value.name,
            email: value.email,
            password: value.password_hash,
            is_admin: value.is_admin,
            points: value.points,
        }
    }
}

/** generic user interface for db (not authenticated)*/
//pub struct Rocket<P: Phase>(pub(crate) P::State);

#[derive(Clone)]
pub struct User<UserType> {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub points: f64,
    pub phantom: PhantomData<UserType>,
}

impl<T: UserType> User<T> {
    //*this must remain private */
    fn upgrade<Z>(self) -> User<Z> {
        User {
            id: self.id,
            name: self.name,
            email: self.email,
            password_hash: self.password_hash,
            is_admin: self.is_admin,
            points: self.points,
            phantom: PhantomData,
        }
    }
}

impl User<Unauthenticated> {
    pub async fn get_by_email(db: &Db, email: String) -> Result<Self, InsignoError> {
        let user: Self = db
            .run(|conn| {
                users::table
                    .filter(users::email.eq(email))
                    .get_result::<UserDiesel>(conn)
            })
            .await
            .map_err(|e| InsignoError::new_debug(404, &e.to_string()))?
            .into();
        Ok(user)
    }
    pub async fn get_by_id(db: &Db, id_user: i64) -> Result<Self, InsignoError> {
        let user: Self = db
            .run(move |conn| {
                users::table
                    .filter(users::id.eq(id_user))
                    .get_result::<UserDiesel>(conn)
            })
            .await
            .map_err(|e| InsignoError::new_debug(404, &e.to_string()))?
            .into();
        Ok(user)
    }
}

impl<T: UserType> User<T> {
    pub async fn insert(&mut self, connection: &Db) -> Result<(), InsignoError> {
        let me: UserDiesel = self.clone().into();
        let mut me: Self = connection
            .run(|conn| {
                insert_into(users::dsl::users)
                    .values::<UserDiesel>(me)
                    .get_result::<UserDiesel>(conn)
            })
            .await
            .map_err(|e| InsignoError::new(422, "impossibile creare l'account", &e.to_string()))?
            .into();
        mem::swap(&mut me, self);
        Ok(())
    }

    pub async fn login(mut v: LoginInfo, connection: &Db) -> Result<Self, InsignoError> {
        v.fmt_password();
        v.fmt_email();

        let user = User::get_by_email(connection, v.email)
            .await
            .map_err(|_| InsignoError::new(401, "invalid user", "invalid user"))?;
        if !user.check_hash(&v.password).await {
            let message = "email o password errati";
            Err(InsignoError::new(403, message, message))
        } else {
            Ok(user.upgrade())
        }
    }
    pub async fn check_hash(&self, password: &str) -> bool {
        let me = self.clone();
        let password = password.to_string();
        spawn_blocking(move || scrypt::scrypt_check(&password, &me.password_hash).unwrap())
            .await
            .unwrap()
    }
}

impl<T: UserType> Serialize for User<T> {
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User<Authenticated> {
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

        let auth: Result<UserDiesel, _> = connection
            .run(move |conn| {
                sql_query("SELECT * FROM autenticate($1, $2);")
                    .bind::<BigInt, _>(id)
                    .bind::<Text, _>(tok)
                    .get_result(conn)
            })
            .await;

        match auth {
            Ok(a) => {
                return request::Outcome::Success(a.into());
            }
            Err(e) => {
                return InsignoError::new(401, "Authentication error", &e.to_string()).into();
            }
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User<AuthenticatedAdmin> {
    type Error = InsignoError;

    async fn from_request(request: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
        let y: Outcome<User<Authenticated>, _> = User::from_request(request).await;
        match y {
            Outcome::Success(x) => {
                if x.is_admin {
                    Outcome::Success(x.upgrade())
                } else {
                    InsignoError::new(401, "Unauthorized", "Unauthorized").into()
                }
            }
            Outcome::Failure(x) => Outcome::Failure(x),
            Outcome::Forward(_) => Outcome::Forward(()),
        }
    }
}
