use rocket::{get, post, form::Form, routes, Route};
use rocket_auth::{Error, Auth, Signup, Login};

#[post("/signup", data="<form>")]
async fn signup(form: Form<Signup>, auth: Auth<'_>) -> Result<&'static str, Error> {
    auth.signup(&form).await?;
    auth.login(&form.into()).await?;
    Ok("You signed up.")
}

#[post("/login", data="<form>")]
async fn login(form: rocket::serde::json::Json<Login>, auth: Auth<'_>) -> Result<&'static str, Error> {
    auth.login(&form).await?;
    Ok("You're logged in.")
}

#[get("/logout")]
fn logout(auth: Auth<'_>)-> Result<(), Error> {
    auth.logout()?;
    Ok(())
}

pub fn get_routes() -> Vec<Route> {
    routes![signup, login, logout]
}