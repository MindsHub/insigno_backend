use std::{collections::BTreeMap, fs};

use diesel::{Connection, PgConnection, RunQueryDsl};
use mail::SmtpConfig;
use rocket::{fairing::*, serde::Deserialize, State};
use rocket_sync_db_pools::Config;
use schema_sql::marker_types;
use utils::InsignoError;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod auth;
mod cors;
mod db;
#[allow(dead_code, unused)]
mod mail;
mod map;
mod pills;
mod schema_rs;
mod schema_sql;
#[cfg(test)]
mod test;
mod utils;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct InsignoConfig {
    media_folder: String,
    oldest_supported_version: String,
    smtp: SmtpConfig,
}
pub struct TrashTypeMap {
    pub to_string: BTreeMap<i64, String>,
    pub to_i64: BTreeMap<String, i64>,
}



pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Insigno config", |rocket| async {
        //generate trash_types_map
        let config = Config::from("db", &rocket).unwrap();

        let mut conn = PgConnection::establish(&config.url).unwrap();
        println!("{:?}", &config.url);
        let sorted = marker_types::table
            .load::<(i64, String, f64)>(&mut conn)
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

#[get("/compatibile?<version_str>")]
async fn compatibile(
    version_str: String,
    config: &State<InsignoConfig>,
) -> Result<String, InsignoError> {
    let supported = config
        .oldest_supported_version
        .trim()
        .split('.')
        .map(|x| x.parse::<i64>().unwrap());
    let test = version_str
        .trim()
        .split('.')
        .map(|x| x.parse::<i64>().unwrap());
    let result = supported.zip(test).fold(None, |prev, (x, y)| {
        if prev.is_some() {
            return prev;
        }
        if x != y {
            Some(x < y)
        } else {
            None
        }
        //todo!();
    });
    if let Some(y) = result {
        Ok(y.to_string())
    } else {
        Ok(true.to_string())
    }
    //todo!()
}

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();
    rocket
        .attach(db::stage())
        .mount("/pills", pills::get_routes())
        .mount("/map", map::get_routes())
        .mount("/", auth::get_routes())
        .mount("/", routes![compatibile])
        .attach(AdHoc::config::<InsignoConfig>())
        .attach(AdHoc::on_ignite("checking config", |rocket| async {
            // if media folder does not exist it creates it
            let cfg: &InsignoConfig = rocket.state().unwrap();

            let _ = fs::create_dir_all(cfg.media_folder.clone());
            rocket
        }))
        .attach(stage())
        .attach(cors::Cors)
        .attach(mail::stage())
        .mount("/", cors::get_routes())
    //.manage(users)
}
