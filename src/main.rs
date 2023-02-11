use rocket::{fairing::AdHoc, serde::Deserialize};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod auth;
mod db;
mod map;
mod pills;
mod schema;
mod utils;
mod cors;
use cors::*;
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
        .mount("/map", map::get_routes())
        .mount("/", auth::get_routes())
        .attach(AdHoc::config::<InsignoConfig>())
        .attach(cors::Cors)
        .mount("/", routes![index, all_options, insert])
    //.manage(users)
}
