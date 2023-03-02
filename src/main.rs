use std::fs;

use rocket::{fairing::*, serde::Deserialize};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod auth;
mod cors;
mod db;
mod map;
mod pills;
mod schema_sql;
mod schema_rs;
mod utils;

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
        .attach(AdHoc::on_ignite("checking config", |rocket| async {
            // if media folder does not exist it creates it
            let cfg :&InsignoConfig =rocket.state().unwrap();
            let _ = fs::create_dir_all(cfg.media_folder.clone());
            rocket
        }))
        .attach(cors::Cors)
        .mount("/", cors::get_routes())
    //.manage(users)
}
