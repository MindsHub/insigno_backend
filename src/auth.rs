//use rocket::form::prelude::Entity::Form;
use diesel::dsl::now;
use diesel::sql_query;
use diesel::sql_types::BigInt;
use diesel::sql_types::Text;

use pbkdf2::pbkdf2_hmac_array;
use serde::Serialize;
use sha2::Sha256;

use rand::distributions::Alphanumeric;
use rand::Rng;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{self, FromRequest};
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

#[derive(FromForm, Deserialize)]
struct CreateInfo {
    name: String,
    password: String,
}

fn hash_password(password: &String) -> String {
    let key = pbkdf2_hmac_array::<Sha256, 20>(password.as_bytes(), "test".as_bytes(), 4096);
    hex::encode(key)
}

fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}

async fn get_user_by_email(db: &Db, email: String) -> Result<User, diesel::result::Error> {
    let users: Vec<User> = db
        .run(|conn| users::table.filter(users::name.eq(email)).get_results(conn))
        .await?;
    //.map_err(to_debug)?;
    let user = users.get(0).ok_or(diesel::result::Error::NotFound)?;
    Ok(user.clone())
}

pub async fn get_user_by_id(db: &Db, id: i64) -> Result<User, diesel::result::Error> {
    let users: Vec<User> = db
        .run(move |conn| users::table.find(id).get_results(conn))
        .await?;
    //.map_err(to_debug)?;
    let user = users.get(0).ok_or(diesel::result::Error::NotFound)?;
    Ok(user.clone())
}

sql_function! {fn autenticate(id_inp: BigInt, tok: Text)->(BigInt, Text, Text, Bool, Double)}

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

#[post("/signup", format = "form", data = "<create_info>")]
async fn signup(
    db: Db,
    mut create_info: Form<CreateInfo>,
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
    login(db, create_info, cookies).await?;
    Ok(Json(user.id.unwrap()))
}

#[post("/login", format = "form", data = "<login_info>")]
async fn login(
    db: Db,
    login_info: Form<CreateInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Json<i64>, InsignoError> {
    let user = get_user_by_email(&db, login_info.name.clone())
        .await
        .map_err(|x| InsignoError::new(401, "email not found", &x.to_string()))?;
    let hash = hash_password(&login_info.password);
    if user.password == hash {
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
        Err(InsignoError::new(
            401,
            "wrong username or password",
            "wrong username or password",
        ))
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
        let response = client.get("/user/1").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }
    #[rocket::async_test]
    async fn test_get_auth_user() {
        test_reset_db();
        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");

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
        let data = "name=IlMagicoTester&password=Testtes1!";
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
