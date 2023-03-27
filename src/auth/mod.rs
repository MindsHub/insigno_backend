use std::fs;

use chrono::Local;
use diesel::dsl::now;

use diesel::query_dsl::methods::FilterDsl;
use diesel::sql_types::Text;
use diesel::{insert_into, sql_query};

use serde::Serialize;

use rocket::form::Form;
use rocket::http::{ContentType, Cookie, CookieJar};
use rocket::serde::json::Json;
use rocket::{Route, State};

use crate::diesel::ExpressionMethods;
use crate::InsignoConfig;
//use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;

use crate::schema_rs::PendingUser;
use crate::schema_sql::user_sessions::dsl::user_sessions;
use crate::schema_sql::user_sessions::{refresh_date, token, user_id};
use crate::schema_sql::{pending_users, users};
use crate::utils::InsignoError;
use crate::{db::Db, schema_rs::User};

pub use self::authentication::*;

mod authentication;
pub mod mail_auth;

#[post("/signup", format = "form", data = "<create_info>")]
async fn signup(
    db: Db,
    mut create_info: Form<SignupInfo>,
    cfg: &State<InsignoConfig>,
) -> Result<String, InsignoError> {
    create_info.check(&db).await?;
    let pending_user: PendingUser = create_info.clone().into();
    let local_time = Local::now();
    pending_user.send_verification_mail(&cfg.smtp)?;

    println!(
        "mail time {}",
        Local::now()
            .signed_duration_since(local_time)
            .num_milliseconds()
    );
    db.run(|conn| {
        insert_into(pending_users::dsl::pending_users)
            .values(pending_user)
            .execute(conn)
    })
    .await
    .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;

    Ok("mail inviata".to_string())
}

#[post("/login", format = "form", data = "<login_info>")]
async fn login(
    db: Db,
    login_info: Form<LoginInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Json<i64>, InsignoError> {
    let user = get_user_by_email(&db, login_info.email.clone())
        .await
        .map_err(|x| InsignoError::new(401, "email not found", &x.to_string()))?;
    if check_hash(&login_info.password, &user.password) {
        let cur_user_id = user.id.unwrap();

        let token_str = generate_token();
        let insigno_auth = format!("{cur_user_id} {token_str}");

        cookies.add_private(Cookie::new("insigno_auth", insigno_auth));

        // update token on login
        db.run(move |conn| {
            diesel::insert_into(user_sessions)
                .values((
                    user_id.eq(cur_user_id),
                    token.eq(token_str.clone()),
                    refresh_date.eq(now),
                ))
                .on_conflict(user_id)
                .do_update()
                .set((token.eq(token_str), refresh_date.eq(now)))
                .execute(conn)
        })
        .await
        .map_err(|x| InsignoError::new(500, "Db Error", &x.to_string()))?;
        Ok(Json(user.id.unwrap()))
    } else {
        Err(InsignoError::new(401, "password errata", "password errata"))
    }
}

#[post("/logout")]
async fn logout(db: Db, cookies: &CookieJar<'_>, user: User) -> Option<()> {
    cookies.remove_private(Cookie::named("insigno_auth"));
    let id = user.id.unwrap();
    if db
        .run(move |conn| diesel::delete(user_sessions.filter(user_id.eq(id))).execute(conn))
        .await
        .is_ok()
    {
        Some(())
    } else {
        None
    }
}

#[post("/session")]
async fn refresh_session(_user: User) -> Option<()> {
    Some(())
}

#[derive(Serialize)]
pub struct UnautenticatedUser {
    id: i64,
    name: String,
    points: f64,
}

impl From<User> for UnautenticatedUser {
    fn from(value: User) -> Self {
        UnautenticatedUser {
            name: value.name,
            points: value.points,
            id: value.id.unwrap(),
        }
    }
}
#[derive(Serialize)]
pub struct AutenticateUser {
    id: i64,
    name: String,
    points: f64,
}

#[get("/user")] //, format="form", data="<login_info>"
fn get_auth_user(user: User) -> Json<AutenticateUser> {
    Json(AutenticateUser {
        id: user.id.unwrap(),
        name: user.name,
        points: user.points,
    })
}

#[get("/user/<id>")] //, format="form", data="<login_info>"
pub async fn get_user(db: Db, id: i64) -> Result<Json<UnautenticatedUser>, InsignoError> {
    let user = get_user_by_id(&db, id)
        .await
        .map_err(|e| InsignoError::new(404, "user not found", &e.to_string()))?;
    Ok(Json(UnautenticatedUser {
        id: user.id.unwrap(),
        name: user.name,
        points: user.points,
    }))
}

#[get("/verify/<cur_token>")]
pub async fn verify(cur_token: String, db: Db) -> Result<(ContentType, String), InsignoError> {
    let pending_user: PendingUser = db
        .run(|conn| {
            sql_query(
                "
        SELECT * FROM get_pending_user($1);",
            )
            .bind::<Text, _>(cur_token)
            .get_result(conn)
            //pending_users::table.filter(pending_users::token.eq(cur_token)).get_result(conn)
        })
        .await
        .map_err(|e| InsignoError::new(422, "token scaduto", &e.to_string()))?;

    let user: User = User {
        id: None,
        name: pending_user.name,
        email: pending_user.email,
        password: pending_user.password_hash,
        is_admin: false,
        points: 0.0,
    };
    db.run(|conn| insert_into(users::dsl::users).values(user).execute(conn))
        .await
        .map_err(|e| InsignoError::new(422, "impossibile creare l'account", &e.to_string()))?;
    let success = fs::read("./templates/account_creation.html")
        .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
    let success =
        String::from_utf8(success).map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
    //let file = NamedFile::open("./templates/account_creation.html").await.unwrap();
    Ok((ContentType::HTML, success))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        login,
        signup,
        logout,
        refresh_session,
        get_auth_user,
        get_user,
        verify,
    ]
}
#[cfg(test)]
mod test {
    use crate::{
        db::Db,
        erase_tables, rocket,
        test::{test_reset_db, test_signup},
    };
    use diesel::RunQueryDsl;
    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
    };
    #[rocket::async_test]
    async fn test_get_user() {
        test_reset_db();
        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");

        //erase_tables!(client, users, user_sessions);

        let response = client.get("/user/1").dispatch().await;
        assert_eq!(response.status(), Status::NotFound);

        test_signup(&client).await;

        // try to get types list
        let response = client.get(format!("/user/1")).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }
    #[rocket::async_test]
    async fn test_get_auth_user() {
        test_reset_db();
        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");

        //erase_tables!(client, users);

        let response = client.get("/user").dispatch().await;
        assert_eq!(response.status(), Status::Unauthorized);
        test_signup(&client).await;

        // try to get types list
        let response = client.get("/user").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }

    #[rocket::async_test]
    async fn test_autentication() {
        test_reset_db();
        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");

        //clean DB
        erase_tables!(client, user_sessions, users);

        // try to get types list
        let data = "name=IlMagicoTester&password=Testtes1!&email=test@test.com";
        let response = client
            .post("/login")
            .header(ContentType::Form)
            .body(data)
            .dispatch();
        assert_eq!(response.await.status(), Status::Unauthorized);

        test_signup(&client).await;

        let response = client
            .post("/session")
            .header(ContentType::Form)
            .body(data)
            .dispatch();
        assert_eq!(response.await.status(), Status::Ok);

        let response = client
            .post("/login")
            .header(ContentType::Form)
            .body(data)
            .dispatch();
        assert_eq!(response.await.status(), Status::Ok);

        let response = client
            .post("/logout")
            //.header(ContentType::Form)
            .body(data)
            .dispatch();
        assert_eq!(response.await.status(), Status::Ok);
    }
}
