use std::error::Error;
//use rocket::form::prelude::Entity::Form;
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use diesel::sql_query;
use diesel::sql_types::BigInt;
use diesel::sql_types::Text;
use diesel::dsl::now;
use rand::Rng;
use rand::distributions::Alphanumeric;
use rocket::Route;
use rocket::request::{FromRequest, self};
use rocket::form::Form;
use rocket::http::{CookieJar, Cookie, Status};
use rocket::serde::json::Json;
use serde::Deserialize;
use rocket::response::Debug;

use crate::diesel::ExpressionMethods;
use crate::diesel::QueryDsl;
use crate::schema_sql::user_sessions::{user_id, token, refresh_date};
use crate::schema_sql::users;
use crate::schema_sql::user_sessions::dsl::user_sessions;
use crate::utils::to_debug;
use crate::{db::Db, schema_rs::User};
use crate::diesel::RunQueryDsl;

#[derive(FromForm, Deserialize)]
struct CreateInfo {
    email: String,
    password: String
}
/*
#[derive(FromForm, Deserialize)]
struct LoginInfo {
    email: String,
    password: String,
}*/

fn hash_password(password: &String) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input_str(password);
    hasher.result_str()
}

fn generate_token()->String{
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}

async fn get_user_by_email(db: &Db, email: String)->Result<User, diesel::result::Error>{
    let users: Vec<User> = db.run(|conn| {
        users::table.filter(users::email.eq(email)).get_results(conn)
        
    }).await?;
    //.map_err(to_debug)?;
    let user= users.get(0).ok_or(diesel::result::Error::NotFound)?;
    Ok(user.clone())
}

sql_function!{fn autenticate(id_inp: BigInt, tok: Text)->(BigInt, Text, Text, Bool, Double)}

#[derive(Responder, Debug)]
pub enum AuthError<T> {
    #[response(status = 401)]
    Unauthorized(T),
}
fn auth_fail(inp: &str)->request::Outcome<User, AuthError<String>>{
    return request::Outcome::Failure((Status::Unauthorized, AuthError::Unauthorized(inp.to_string())));
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error=AuthError<String>;

    async fn from_request(request: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
        let connection =  request.guard::<Db>().await.unwrap();
        let cookie = request.cookies();
        let id: i64 = match cookie.get_private("user_id"){
            Some(a) => {println!("{:?}", a.value()); a},
            None => {return auth_fail("user_id cookie not found");}
        }.value().parse().unwrap();

        let tok = match cookie.get_private("token"){
            Some(a) => {println!("{:?}", a.value()); a},
            None => {return auth_fail("token cookie not found");}
        }.value().to_string();

        //let tmpTok = tok.clone();

        let auth: Result<User, _>  = connection.run(move |conn|{
            sql_query(&format!("SELECT * FROM autenticate({id}, '{tok}');", ))
            .get_result(conn)
        }).await;
        //println!("{auth:?} {}", format!("SELECT * FROM autenticate({id}, '{tmpTok}');"));
        //todo!();
        match auth{
            Ok(a) => {return request::Outcome::Success(a);},
            Err(_) => {return  auth_fail("errore nell'autenticazione");},
        }
    }
}

#[post("/signup", format="form", data="<create_info>")]
async fn signup(db: Db, create_info: Form<CreateInfo>, cookies: &CookieJar<'_>)
  -> Result<Json<i64>, Debug<Box<dyn Error>>> {
    let user: User = User{
        id: None,
        email: create_info.email.clone(),
        password: hash_password(&create_info.password.clone()),
        points: 0.0,
        is_admin: false,
    };
    let user: User = db.run(|conn| {
        diesel::insert_into(users::table)
        .values(user)
        .get_result(conn)
    }).await
        .map_err(to_debug)?;
    login(db, create_info, cookies).await?;
    Ok(Json(user.id.unwrap()))
}

#[post("/login", format="form", data="<login_info>")]
async fn login(db:Db, login_info: Form<CreateInfo>, cookies: &CookieJar<'_>)-> Result<Option<()>, Debug<Box<dyn Error>>>{
    let user = get_user_by_email(&db, login_info.email.clone())
        .await;
    let user = match user{
        Ok(a) => {a},
        Err(_) => {return Ok(None);}
    };
    let hash = hash_password(&login_info.password);
    if user.password==hash{
        let cur_user_id = user.id.unwrap().clone();
        
        let token_str= generate_token();
        cookies.add_private(Cookie::new("user_id", user.id.unwrap().to_string()));
        cookies.add_private(Cookie::new("token", token_str.clone()));
        // update token on login
        db.run(move |conn| {
            diesel::insert_into(user_sessions)
                .values((user_id.eq(cur_user_id), token.eq(token_str.clone()), refresh_date.eq(now)))
                .on_conflict(user_id)
                .do_update()
                .set((token.eq(token_str), refresh_date.eq(now)))
                .execute(conn)
        }).await
        .map_err(to_debug)?;
        Ok(Some(()))
    }else{
        Ok(None)
    }
}

#[post("/logout")]
async fn logout(db: Db, cookies: &CookieJar<'_>, user: User)->Option<()>{
    cookies.remove_private(Cookie::named("user_id"));
    cookies.remove_private(Cookie::named("token"));
    let id = user.id.unwrap();
    if db.run(move |conn| {
        let tmp =diesel::delete(user_sessions.filter(user_id.eq(id))).execute(conn);
        tmp
    }).await.is_ok(){
        Some(())
    }else{
        None
    }
}

#[post("/session")]
async fn refresh_session(_user: User)->Option<()>{
    Some(())
}

pub fn get_routes()-> Vec<Route>{
    routes![
        login,
        signup,
        logout,
        refresh_session,
    ]
}