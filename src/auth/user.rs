use std::marker::PhantomData;

use crate::auth::scrypt::scrypt_check;
use crate::diesel::query_dsl::methods::FilterDsl;
use crate::diesel::ExpressionMethods;
use crate::schema_sql::user_sessions::dsl::user_sessions;
use crate::schema_sql::user_sessions::*;
use crate::{db::Db, utils::InsignoError};
use chrono::Utc;
use diesel::dsl::now;
use diesel::sql_types::{BigInt, Text};
use diesel::RunQueryDsl;
use diesel::{insert_into, sql_query};
use rocket::request::{self, FromRequest, Outcome};
use rocket::tokio::task::spawn_blocking;
use serde::ser::SerializeStruct;
use serde::Serialize;

table! {
    users(id){
        id -> Nullable<BigInt>,
        name -> Text,
        email -> Text,
        password -> Text,
        is_admin -> Bool,
        points -> Double,
        accepted_to_review -> Nullable<Bool>,
        last_revision-> Nullable<Timestamptz>,
    }
}

#[derive(Clone)]
pub struct YesReview;
#[derive(Clone)]
pub struct AnyReview;
pub trait UserReview: Clone + Send {
    fn requires_accepted_to_review() -> bool;
}
impl UserReview for YesReview {
    fn requires_accepted_to_review() -> bool {
        true
    }
}
impl UserReview for AnyReview {
    fn requires_accepted_to_review() -> bool {
        false
    }
}

#[derive(Clone)]
pub struct Unauthenticated;

#[derive(Clone)]
pub struct Authenticated;

#[derive(Clone)]
pub struct AuthenticatedAdmin;

pub trait UserType: Clone + Send {}
impl UserType for Unauthenticated {}
impl UserType for Authenticated {}
impl UserType for AuthenticatedAdmin {}

#[derive(Insertable, Queryable, QueryableByName, AsChangeset)]
#[diesel(table_name = users)]
pub(crate) struct UserDiesel {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub is_admin: bool,
    pub points: f64,
    pub accepted_to_review: Option<bool>,
    pub last_revision: Option<chrono::DateTime<Utc>>,
}

#[derive(Clone)]
pub struct User<UserType, UserAge = AnyReview> {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub points: f64,
    pub accepted_to_review: Option<bool>,
    pub last_revision: Option<chrono::DateTime<Utc>>,

    pub phantom: PhantomData<UserType>,
    pub phantom_age: PhantomData<UserAge>,
}

impl<T: UserType, R: UserReview> TryFrom<UserDiesel> for User<T, R> {
    type Error = InsignoError;
    fn try_from(value: UserDiesel) -> Result<User<T, R>, Self::Error> {
        if value.accepted_to_review != Some(true) && R::requires_accepted_to_review() {
            Err(InsignoError::new(403).both("you did not accept to review"))
            //when is not an adult, and it ask for an adult
        } else {
            Ok(Self {
                id: value.id,
                name: value.name,
                email: value.email,
                password_hash: value.password,
                is_admin: value.is_admin,
                points: value.points,
                last_revision: value.last_revision,
                accepted_to_review: value.accepted_to_review,
                phantom: PhantomData,
                phantom_age: PhantomData,
            })
        }
    }
}

/** generic user interface for db (not authenticated)*/
//pub struct Rocket<P: Phase>(pub(crate) P::State);

impl<T: UserType> User<T> {
    //*this must remain private */
    // we just ignore age, because even with an update it is the same
    fn upgrade<Z>(self) -> User<Z> {
        User {
            id: self.id,
            name: self.name,
            email: self.email,
            password_hash: self.password_hash,
            is_admin: self.is_admin,
            points: self.points,
            last_revision: self.last_revision,
            accepted_to_review: self.accepted_to_review,
            phantom: PhantomData,
            phantom_age: PhantomData,
        }
    }
}

impl<Age: UserReview> User<Authenticated, Age> {
    pub async fn set_token(&self, token_str: &str, db: &Db) -> Result<(), InsignoError> {
        let id = self.get_id();
        let token_str = token_str.to_string();
        db.run(move |conn| {
            diesel::insert_into(user_sessions)
                .values((
                    user_id.eq(id),
                    token.eq(token_str.clone()),
                    refresh_date.eq(now),
                ))
                .on_conflict(user_id)
                .do_update()
                .set((token.eq(token_str), refresh_date.eq(now)))
                .execute(conn)
        })
        .await
        .map_err(|e| InsignoError::new(500).debug(e))?;
        Ok(())
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
            .map_err(|e| InsignoError::new(404).debug(e))?
            .try_into()?;
        Ok(user)
    }
    pub async fn get_by_id(db: &Db, id_user: i64) -> Result<Self, InsignoError> {
        UserDiesel::get_by_id(db, id_user)
            .await? //
            .try_into()
    }
    pub async fn login(self, password: &str) -> Result<User<Authenticated>, InsignoError> {
        if !self.check_hash(password).await {
            Err(InsignoError::new(403).client("email o password errati"))
        } else {
            let me = self.upgrade();
            Ok(me)
        }
    }
    pub async fn check_hash(&self, password: &str) -> bool {
        let me = self.clone();
        let password = password.to_string();
        spawn_blocking(move || scrypt_check(&password, &me.password_hash).unwrap())
            .await
            .unwrap()
    }
}

impl UserDiesel {
    pub async fn get_by_id(connection: &Db, id_user: i64) -> Result<Self, InsignoError> {
        connection
            .run(move |conn| {
                users::table
                    .filter(users::id.eq(id_user))
                    .get_result::<UserDiesel>(conn)
            })
            .await
            .map_err(|e| InsignoError::new(404).debug(e))
    }
    pub async fn insert(self, connection: &Db) -> Result<(), InsignoError> {
        connection
            .run(|conn| {
                insert_into(users::dsl::users)
                    .values::<UserDiesel>(self)
                    .execute(conn)
            })
            .await
            .map(|_| ())
            .map_err(|e| {
                InsignoError::new(422)
                    .client("impossibile creare l'account")
                    .debug(e)
            })
    }

    pub async fn update(self, connection: &Db) -> Result<(), InsignoError> {
        connection
            .run(|conn| {
                diesel::update(users::dsl::users)
                    .filter(users::id.eq(self.id))
                    .set(self)
                    .execute(conn)
            })
            .await
            .map(|_| ())
            .map_err(|e| InsignoError::new(500).debug(e))
    }
}

impl<T: UserType, R: UserReview> User<T, R> {
    pub fn get_id(&self) -> i64 {
        self.id.unwrap()
    }
}

impl<R: UserReview> Serialize for User<Authenticated, R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("User", 5)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("points", &self.points)?;
        s.serialize_field("is_admin", &self.is_admin)?;
        s.serialize_field("email", &self.email)?;
        s.serialize_field("accepted_to_review", &self.accepted_to_review)?;
        s.serialize_field("last_revision", &self.last_revision)?;
        s.end()
    }
}
impl Serialize for User<Unauthenticated> {
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
impl<'r, Age: UserReview> FromRequest<'r> for User<Authenticated, Age> {
    type Error = InsignoError;
    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let connection = request.guard::<Db>().await.unwrap();
        let cookie = request.cookies();
        let insigno_auth_cookie = match cookie.get_private("insigno_auth") {
            Some(a) => a,
            None => {
                return InsignoError::new(401)
                    .both("insigno_auth cookie not found")
                    .into();
            }
        };
        let insigno_auth = insigno_auth_cookie.value().to_string();
        let vec: Vec<&str> = insigno_auth.split(' ').collect();

        let id: i64 = vec[0].parse().unwrap();
        let tok = vec[1].to_string();
        if !tok.chars().all(|x| x.is_ascii_alphanumeric()) {
            return InsignoError::new(401).both("sql injection?").into();
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
                return request::Outcome::Success(match a.try_into() {
                    Ok(a) => a,
                    Err(x) => {
                        return x.into();
                    }
                });
            }
            Err(e) => {
                cookie.remove_private(insigno_auth_cookie);
                return InsignoError::new(401)
                    .client("Authentication error")
                    .debug(e)
                    .into();
            }
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User<AuthenticatedAdmin> {
    type Error = InsignoError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let y: Outcome<User<Authenticated>, _> = User::from_request(request).await;
        match y {
            Outcome::Success(x) => {
                if x.is_admin {
                    Outcome::Success(x.upgrade())
                } else {
                    InsignoError::new(401).both("Unauthorized").into()
                }
            }
            Outcome::Failure(x) => Outcome::Failure(x),
            Outcome::Forward(_) => Outcome::Forward(()),
        }
    }
}
