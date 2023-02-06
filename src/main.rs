use rocket::{serde::json::Json, fairing::AdHoc, serde::Deserialize};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod auth;
mod db;
mod pills;
mod trash;
mod utils;
mod schema;

#[get("/test")]
fn test() -> Json<String> {
    Json("ok".to_string())
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct InsignoConfig {
    media_folder: String,
}


#[launch]
async fn rocket() -> _ {
    let rocket = rocket::build();


    rocket
        .attach(db::stage())
        .attach(auth::stage().await)
        .mount("/pills", pills::get_routes())
        .mount("/", routes![test])
        .mount("/trash", trash::get_routes())
        .mount("/", auth::get_routes())
        .attach(AdHoc::config::<InsignoConfig>())
    //.manage(users)
}
