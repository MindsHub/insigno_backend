
use diesel::dsl::now;

use serde::Serialize;

use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;
use rocket::Route;
use serde::Deserialize;

use crate::diesel::ExpressionMethods;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::schema_sql::user_sessions::dsl::user_sessions;
use crate::schema_sql::user_sessions::{refresh_date, token, user_id};
use crate::schema_sql::users;
use crate::utils::InsignoError;
use crate::{db::Db, schema_rs::User};

pub use self::authentication::*;

mod authentication;

#[derive(FromForm, Deserialize)]
struct SignupInfo {
    name: String,
    email: String,
    password: String,
}

#[derive(FromForm, Deserialize)]
struct LoginInfo {
    email: String,
    password: String,
}
impl From<SignupInfo> for LoginInfo{
    fn from(value: SignupInfo) -> Self {
        Self { email: value.email, password: value.password }
    }
}

#[post("/signup", format = "form", data = "<create_info>")]
async fn signup(
    db: Db,
    mut create_info: Form<SignupInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Json<i64>, InsignoError> {
    create_info.name = create_info.name.trim().to_string();
    let name_len = create_info.name.len();
    if name_len < 3 && 20 < name_len {
        let message = "Nome utente invalido. Deve essere lungo tra 3 e 20 caratteri (e possibilmente simile al nome)";
        return Err(InsignoError::new(401, message, message));
    }
    if !create_info
        .name
        .chars()
        .all(|x| x.is_alphanumeric() || x == '_' || x == ' ')
    {
        let message =
            "Nome utente invalido. Un nome corretto può contenere lettere, numeri, spazi e _";
        return Err(InsignoError::new(401, message, message));
    }
    if create_info.password.len() < 8 {
        let message = "Password troppo breve, deve essere lunga almeno 8 caratteri";
        return Err(InsignoError::new(401, message, message));
    }
    if !create_info.password.chars().any(|x| x.is_ascii_uppercase()) {
        let message = "La password deve contenere almeno una maiuscola";
        return Err(InsignoError::new(401, message, message));
    }
    if !create_info.password.chars().any(|x| x.is_ascii_lowercase()) {
        let message = "La password deve contenere almeno una minuscola";
        return Err(InsignoError::new(401, message, message));
    }
    if !create_info.password.chars().any(|x| x.is_numeric()) {
        let message = "La password deve contenere almeno un numero";
        return Err(InsignoError::new(401, message, message));
    }
    

    if !create_info
        .password
        .chars()
        .any(|x| !x.is_ascii_alphanumeric())
    {
        let message = "La password deve contenere almeno un carattere speciale";
        return Err(InsignoError::new(401, message, message));
    }

    let user: User = User {
        id: None,
        name: create_info.name.clone(),
        email: create_info.email.clone(),
        password: hash_password(&create_info.password.clone()),
        points: 0.0,
        is_admin: false,
    };

    let user: User = db
        .run(|conn| {
            diesel::insert_into(users::table)
                .values(user)
                .get_result(conn)
        })
        .await
        .map_err(|x| InsignoError::new(401, "Nome utente usato", &format!("{x:?}")))?;
    login(db, LoginInfo::from(create_info.into_inner()).into(), cookies).await?;
    Ok(Json(user.id.unwrap()))
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

pub fn get_routes() -> Vec<Route> {
    routes![
        login,
        signup,
        logout,
        refresh_session,
        get_auth_user,
        get_user,
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

        let id = test_signup(&client).await;

        // try to get types list
        let response = client.get(format!("/user/{id}")).dispatch().await;
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
        let data = "email=test@test.com&password=Testtes1!";
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
