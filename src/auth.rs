use std::collections::BTreeMap;
use std::error::Error;

use crate::schema_rs::*;
use diesel::*;
use diesel::{insert_into, update, Connection, PgConnection, QueryDsl, RunQueryDsl};
use rocket::response::Debug;
use rocket::serde::json::{from_str, serde_json, Json};
use rocket::{fairing::AdHoc, form::Form, get, post, routes, Route};
use rocket_auth::User as AUser;
use rocket_auth::{Auth, DBConnection, Login, Result, Session, Signup, Users};
use rocket_sync_db_pools::Config;

use crate::schema_sql::{marker_types, users};
use crate::utils::*;
pub struct UserConnection(pub diesel::PgConnection);
unsafe impl Sync for UserConnection {}

#[rocket::async_trait]
impl DBConnection for UserConnection {
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    async fn create_user(&self, email: &str, hash: &str, is_admin: bool) -> Result<()> {
        let email = email.to_string();
        let hash = hash.to_string();

        use crate::diesel::ExpressionMethods;
        use users::dsl::users as dslUsers;
        insert_into(dslUsers)
            .values((
                users::email.eq(email),
                users::password.eq(hash),
                users::is_admin.eq(is_admin),
            ))
            .execute(&self.0)?;
        Ok(())
    }

    async fn update_user(&self, user: &AUser) -> Result<()> {
        let user = user.clone();

        use users::dsl::users as dslUsers;

        update(dslUsers.find(user.id as i64))
            .set((
                users::email.eq(user.email().to_string()),
                users::password.eq(user.password),
                users::is_admin.eq(user.is_admin),
            ))
            .execute(&self.0)?;

        Ok(())
    }

    async fn delete_user_by_id(&self, user_id: i32) -> Result<()> {
        use users::dsl::users as dslUsers;
        delete(dslUsers.find(user_id as i64)).execute(&self.0)?;

        Ok(())
    }
    async fn delete_user_by_email(&self, email: &str) -> Result<()> {
        let email = email.to_string();
        use users::dsl::users as dslUsers;
        delete(dslUsers)
            .filter(users::email.eq(email))
            .execute(&self.0)?;
        Ok(())
    }
    async fn get_user_by_id(&self, user_id: i32) -> Result<AUser> {
        println!("get user");
        use users::dsl::users as dslUsers;
        let z = dslUsers.find(user_id as i64).load::<User>(&self.0)?[0].clone();
        let y: Result<AUser> = Ok(z.clone().into());
        println!("{y:?}");
        y
    }
    async fn get_user_by_email(&self, email: &str) -> Result<AUser> {
        println!("get user by email /{email}/");
        let email = email.to_string();
        use users::dsl::users as dslUsers;
        let z = dslUsers
            .filter(users::email.eq(email))
            .first::<User>(&self.0);
        println!("{}", z.is_err());
        Ok(z?.into())
    }
}

#[post("/signup", data = "<form>")]
async fn signup(form: Form<Signup>, auth: Auth<'_>) -> Result<&'static str, Debug<Box<dyn Error>>> {
    auth.signup(&form).await.map_err(to_debug)?;
    auth.login(&form.into()).await.map_err(to_debug)?;
    Ok("You signed up.")
}
use rocket::serde::Serialize;
#[derive(Serialize)]
struct Token {
    token: String,
}
#[post("/login", data = "<form>")]
async fn login(
    form: Json<Login>,
    auth: Auth<'_>,
) -> Result<Json<rocket::serde::json::Value>, Debug<Box<dyn Error>>> {
    auth.login(&form).await.map_err(to_debug)?;
    //println!("{:?}, {:?}", &form, &form.password);
    let session = auth
        .cookies
        .get_pending("rocket_auth")
        .ok_or(str_to_debug("failed to get cookies"))?;
    let y: Session = from_str(session.value()).map_err(to_debug)?;

    let js = serde_json::json!(Token {
        token: y.auth_key.clone()
    });
    println!("{}", y.auth_key);
    Ok(Json(js))
}

#[get("/logout")]
fn logout(auth: Auth<'_>) -> Result<(), Debug<Box<dyn Error>>> {
    auth.logout().map_err(to_debug)?;
    Ok(())
}

pub fn get_routes() -> Vec<Route> {
    routes![signup, login, logout]
}

pub struct TrashTypeMap {
    pub to_string: BTreeMap<i64, String>,
    pub to_i64: BTreeMap<String, i64>,
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel Authentication Stage", |rocket| async {
        let config = Config::from("db", &rocket).unwrap();
        let conn = PgConnection::establish(&config.url).unwrap();
        println!("{}", config.url);
        let tmp = marker_types::table
        .load::<(i64, String, f32)>(&conn);

        println!("{:?}", tmp);
        let sorted = marker_types::table
            .load::<(i64, String, f32)>(&conn)
            .unwrap()
            .into_iter()
            .map(|(x, y, ..)| (x, y))
            .collect::<BTreeMap<i64, String>>();
        let inverted = sorted.clone().into_iter().map(|(x, y)| (y, x)).collect();
        let trash_types_map = TrashTypeMap {
            to_string: sorted,
            to_i64: inverted,
        };
        let users: Users = UserConnection { 0: conn }.into();

        rocket.manage(users).manage(trash_types_map)
    })
}
