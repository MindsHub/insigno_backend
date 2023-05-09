use rocket::tokio::fs;

use diesel::dsl::now;

use diesel::query_dsl::methods::FilterDsl;

use serde::Serialize;

use rocket::form::Form;
use rocket::http::{ContentType, Cookie, CookieJar};
use rocket::serde::json::Json;
use rocket::{Route, State};

use crate::auth::login_info::LoginInfo;
use crate::auth::signup_info::SignupInfo;
use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;

use crate::db::Db;
use crate::mail::Mailer;
use crate::schema_sql::user_sessions::dsl::user_sessions;
use crate::schema_sql::user_sessions::{refresh_date, token, user_id};
use crate::utils::InsignoError;

use self::authenticated_user::AuthenticatedUser;
pub use self::pending_user::*;
use self::user::User;
pub mod admin_user;
pub mod authenticated_user;
pub mod login_info;
pub mod pending_user;
pub mod signup_info;
pub mod user;
pub mod validation;
pub mod scrypt;
/*
signup info -> pending user (verifica credenziali) #
pending user -> email + db (inviare la mail e salvarla nel db)
pending user -> user (finire registrazione)
login info->  auth-user/admin-auth-user
cookie -> auth-user/admin-auth-user*/

#[post("/signup", format = "form", data = "<create_info>")]
async fn signup(
    db: Db,
    create_info: Form<SignupInfo>,
    mail_cfg: &State<Mailer>,
) -> Result<String, InsignoError> {
    //check if all values are correct
    let pending = PendingUser::new(create_info.into_inner(), &db).await?;

    //send registration mail and insert it in db
    pending.register_and_mail(&db, mail_cfg).await?;

    Ok("mail inviata".to_string())
}

#[get("/verify/<cur_token>")]
pub async fn verify(
    cur_token: String,
    connection: Db,
) -> Result<(ContentType, String), InsignoError> {
    let pending_user = PendingUser::from_token(cur_token, &connection).await?;
    //inserting into db
    User::new_from_pending(pending_user, &connection).await?;

    let success = fs::read("./templates/account_creation.html")
        .await
        .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;

    let success =
        String::from_utf8(success).map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;

    Ok((ContentType::HTML, success))
}

#[post("/login", format = "form", data = "<login_info>")]
async fn login(
    db: Db,
    login_info: Form<LoginInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Json<i64>, InsignoError> {
    let user = User::login(login_info.into_inner(), &db).await?;

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
}

#[post("/logout")]
async fn logout(db: Db, cookies: &CookieJar<'_>, user: AuthenticatedUser) -> Option<()> {
    cookies.remove_private(Cookie::named("insigno_auth"));
    let id = user.as_ref().id.unwrap();
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
fn refresh_session(_user: AuthenticatedUser) -> Option<()> {
    Some(())
}

#[derive(Serialize)]
pub struct AutenticatedUserTest {
    id: i64,
    name: String,
    points: f64,
}

#[get("/user")] //, format="form", data="<login_info>"
fn get_auth_user(user: AuthenticatedUser) -> Json<AuthenticatedUser> {
    Json(user)
}

#[get("/user/<id>")] //, format="form", data="<login_info>"
pub async fn get_user(db: Db, id: i64) -> Result<Json<User>, InsignoError> {
    let user = User::get_by_id(&db, id).await?;
    Ok(Json(user))
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
        rocket,
        test::{test_reset_db, test_signup},
    };
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
