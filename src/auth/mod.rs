/*!
# Authentication black-magic 🪄
We want a customizzable user that support password-based authentication and mail verification

 */
use diesel::query_dsl::methods::FilterDsl;

use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;
use rocket::Route;

use crate::auth::user::{Authenticated, Unauthenticated};
use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;

use crate::db::Db;
use crate::schema_sql::user_sessions::dsl::user_sessions;
use crate::schema_sql::user_sessions::user_id;
use crate::utils::InsignoError;

use self::user::User;

pub mod change_password;
pub mod delete_account;
pub mod login;
pub mod scrypt;
pub mod signup;
pub mod user;
pub mod validation;

/**
# When a user want's to logout, it calls this function
It takes:
 - User cookies (for clearing our authentication token)
 - Authenticated user dataguard (unauthenticated users SHALL NOT PASS! 🧙‍♂️)
 - a Db connection (we must remove the token from user_session)
 */
#[post("/logout")]
pub async fn logout(
    cookies: &CookieJar<'_>,
    user: Result<User<Authenticated>, InsignoError>,
    db: Db,
) -> Result<(), InsignoError> {
    let user = user?;
    cookies.remove_private(Cookie::from("insigno_auth"));
    let id = user.get_id();
    if db
        .run(move |conn| diesel::delete(user_sessions.filter(user_id.eq(id))).execute(conn))
        .await
        .is_ok()
    {
        Ok(())
    } else {
        Err(InsignoError::new(401).client("you are not logged in"))
    }
}

/**
# Refresh session token
It needs only an authenticated user dataguard, that already refreshes the token
 */
#[post("/session")]
fn refresh_session(user: Result<User<Authenticated>, InsignoError>) -> Result<(), InsignoError> {
    let _user = user?;
    //<>
    Ok(())
}

/**
# Get authenticated user information.
It uses the deserialize implementation
 */
#[get("/user")] //, format="form", data="<login_info>"
fn get_auth_user(user: User<Authenticated>) -> Json<User<Authenticated>> {
    Json(user)
}

/**
# Get user information for another user.
It uses the deserialize implementation
 */
#[get("/user/<id>")] //, format="form", data="<login_info>"
pub async fn get_user(db: Db, id: i64) -> Result<Json<User<Unauthenticated>>, InsignoError> {
    let user = User::get_by_id(&db, id).await?;
    Ok(Json(user))
}

/**
# Map routes for Rocket
 */
pub fn get_routes() -> Vec<Route> {
    routes![
        login::login,
        signup::signup,
        logout,
        refresh_session,
        get_auth_user,
        get_user,
        change_password::change_password,
        delete_account::delete_account,
        delete_account::delete_account_web,
        delete_account::delete_account_web_form,
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
        let response = client.get(("/user/1").to_string()).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        let message = response.into_string().await.unwrap();
        assert_eq!(message, r#"{"id":1,"name":"IlMagicoTester","points":0.0}"#);
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
        let message = response.into_string().await.unwrap();
        assert_eq!(
            message,
            r#"{"id":1,"name":"IlMagicoTester","points":0.0,"is_admin":false,"email":"test@test.com","accepted_to_review":null,"last_revision":"2023-04-01T12:00:00Z"}"#
        );
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
