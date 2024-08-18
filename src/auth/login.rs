use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    serde::json::Json,
    State,
};

use crate::{db::Db, pending::generate_token, utils::InsignoError};

use super::{
    user::{Authenticated, User},
    validation::{Email, Password, SanitizeEmail, SanitizePassword, ScryptSemaphore},
};

#[derive(FromForm)]
pub struct LoginInfo {
    pub email: String,
    pub password: String,
}
impl Email for LoginInfo {
    fn get_email(&mut self) -> &mut String {
        &mut self.email
    }
}
impl Password for LoginInfo {
    fn get_password(&mut self) -> &mut String {
        &mut self.password
    }
}

impl LoginInfo {
    fn sanitize(&mut self) -> Result<(), InsignoError> {
        self.sanitize_email()
            .map_err(|x| InsignoError::new(401).both(x))?;
        self.fmt_password();
        Ok(())
    }

    pub async fn into_authenticated_user(
        mut self,
        scrypt_sem: &State<ScryptSemaphore>,
        connection: &Db,
    ) -> Result<User<Authenticated>, InsignoError> {
        self.sanitize()?;
        let user = User::get_by_email(connection, self.email)
            .await
            .map_err(|_| InsignoError::new(401).both("not found"))?;
        let y = scrypt_sem.aquire().await?;
        let user = user.login(&self.password, scrypt_sem).await?; //this is not hashed
        drop(y);
        Ok(user)
    }
}

#[post("/login", format = "form", data = "<login_info>")]
pub async fn login(
    login_info: Form<LoginInfo>,
    cookies: &CookieJar<'_>,
    scrypt_sem: &State<ScryptSemaphore>,
    connection: Db,
) -> Result<Json<i64>, InsignoError> {
    let user = login_info
        .into_inner()
        .into_authenticated_user(scrypt_sem, &connection)
        .await?;

    let token_str = generate_token();
    let insigno_auth = format!("{} {token_str}", user.get_id());
    cookies.add_private(Cookie::new("insigno_auth", insigno_auth));

    // update token
    user.set_token(&token_str, &connection).await?;
    Ok(Json(user.get_id()))
}
