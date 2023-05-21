use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    serde::json::Json,
};
use serde::Serialize;

use crate::{db::Db, pending::generate_token, utils::InsignoError};

use super::{
    user::User,
    validation::{Email, Password, SanitizeEmail, SanitizePassword},
};

#[derive(Serialize, FromForm)]
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
    pub fn sanitize(&mut self) -> Result<(), InsignoError> {
        self.sanitize_email()
            .map_err(|x| InsignoError::new(401, x, x))?;
        self.sanitize_password()
            .map_err(|x| InsignoError::new(401, x, x))?;
        Ok(())
    }
}

#[post("/login", format = "form", data = "<login_info>")]
pub async fn login(
    db: Db,
    mut login_info: Form<LoginInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Json<i64>, InsignoError> {
    //let user = User::login(login_info.into_inner(), &db).await?;
    login_info.sanitize()?;
    let user = User::get_by_email(&db, login_info.email.clone())
        .await
        .map_err(|_| InsignoError::new(401, "not found", "not found"))?;
    let user = user.login(&login_info.password).await?; //this is not hashed

    let token_str = generate_token();
    let insigno_auth = format!("{} {token_str}", user.id.unwrap());

    cookies.add_private(Cookie::new("insigno_auth", insigno_auth));

    // update token
    user.set_token(&token_str, &db).await?;
    Ok(Json(user.id.unwrap()))
}
