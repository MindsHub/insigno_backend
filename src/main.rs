use rocket::serde::json::Json;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod auth;
mod db;
mod pills;
mod trash;
mod utils;

#[get("/test")]
fn test() -> Json<String> {
    Json("ok".to_string())
}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .attach(db::stage())
        .attach(auth::stage().await)
        .mount("/pills", pills::get_routes())
        .mount("/", routes![test])
        .mount("/trash", trash::get_routes())
        .mount("/", auth::get_routes())

    //.manage(users)
}
