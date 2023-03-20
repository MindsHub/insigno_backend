use std::{collections::BTreeMap, fs};

use diesel::{Connection, PgConnection, RunQueryDsl};
use rocket::{fairing::*, serde::Deserialize};
use rocket_sync_db_pools::Config;
use schema_sql::marker_types;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod auth;
mod cors;
mod db;
mod map;
mod pills;
mod schema_rs;
mod schema_sql;
#[cfg(test)]
mod test;
mod utils;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct InsignoConfig {
    media_folder: String,
}
pub struct TrashTypeMap {
    pub to_string: BTreeMap<i64, String>,
    pub to_i64: BTreeMap<String, i64>,
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Insigno config", |rocket| async {
        //generate trash_types_map
        let config = Config::from("db", &rocket).unwrap();
        
        let conn = PgConnection::establish(&config.url).unwrap();
        //std::env::set_var("DATABASE_URL", config.url);
        let sorted = marker_types::table
            .load::<(i64, String, f64)>(&conn)
            .unwrap()
            .into_iter()
            .map(|(x, y, ..)| (x, y))
            .collect::<BTreeMap<i64, String>>();
        let inverted = sorted.clone().into_iter().map(|(x, y)| (y, x)).collect();
        let trash_types_map = TrashTypeMap {
            to_string: sorted,
            to_i64: inverted,
        };

        rocket.manage(trash_types_map)
    })
}

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();
    rocket
        .attach(db::stage())
        .mount("/pills", pills::get_routes())
        .mount("/map", map::get_routes())
        .mount("/", auth::get_routes())
        .attach(AdHoc::config::<InsignoConfig>())
        .attach(AdHoc::on_ignite("checking config", |rocket| async {
            // if media folder does not exist it creates it
            let cfg: &InsignoConfig = rocket.state().unwrap();
            let _ = fs::create_dir_all(cfg.media_folder.clone());
            rocket
        }))
        .attach(stage())
        .attach(cors::Cors)
        .mount("/", cors::get_routes())
    //.manage(users)
}
