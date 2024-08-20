/*! Welcome to INSIGNO, an app for taking care of the environment while having fun.
This is our backend service for managing all the request that our app needs.
For code management reasons, we split our codebase in different modules, each one in charge of a single app aspect.
In particular:
- [self]: Sticks all the modules together!
- [schema_sql]: Defines our database structure(needed by diesel, probably we will remove that file in a future release)
- [schema_rs]: Rust counterpart of schema_sql. This file will be DEFINITELY removed in a future release
- [cors]: Handles all cors request
- [db]: It connects to our postgres with diesel and rocket_sync_db_pool.
- [mail]: Send super cool (html) mail using lettre
- [pending]: Handles all the different types of pending request that we will possibly ever need (for now mail-verification), and forward them to the correct handler
- [pills]: Manages our super interesting pills.
- [utils]: Manages some utility used in all the crate. Smaller this file is, the better.
- [auth]: Signup, login, account verification...
- [map]: Marker handling
- [test]: Defines some methods used for testing all around the crate.

In addition to that in this crate you could find the test script.
some command's that you should run before using it.
- `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` install rustup
- `sudo apt install docker.io` in test scrypt we use docker
- `cargo install cargo-watch cargo-tarpaulin` install some cargo cool thing
- `cargo install diesel-cli --no-default-features --features "postgres"`

Stylistic Roadmap:
- [x] Remove italian comments
- [ ] DOCUMENTING
- [ ] TESTING
- [ ] Remove utils, schema_sql and schema_rs
- [ ] Split container in multiple independent crate (faster compilation and better organization)

Roadmap of the next implementations:
- [x] Login/Signup
- [x] Change password
- [ ] Localization (server side? app side?)
- [ ] Manage groups of users
- [ ] Cool way to assign points, we want to boost responsible use of the app
- [ ] Mitigate ddos attacks
- [ ] From Insigno.toml it should be straightforward to implement custom server
- [ ] Dockerize, and keep it easily scalable
- [ ] Better pgsql query, now we prefer to use raw diesel and implement function db-side

*/
use std::sync::Arc;
use std::{collections::BTreeMap, fs};

use auth::scrypt::InsignoScryptParams;
use auth::validation::ScryptSemaphore;
use mail::SmtpConfig;
use prometheus::process_collector::ProcessCollector;
use rocket::config::Config;
use rocket::tokio::sync::Semaphore;
use rocket::{
    fairing::*,
    figment::{
        providers::{Env, Format, Serialized, Toml},
        Figment,
    },
    serde::Deserialize,
    State,
};
use rocket_prometheus::PrometheusMetrics;

use utils::InsignoError;

#[macro_use]
pub extern crate rocket;
#[macro_use]
extern crate diesel;

pub mod auth;
mod cors;
pub mod db;
mod files;
#[allow(dead_code, unused)]
mod mail;
mod map;
mod pending;
mod pills;
mod schema_rs;
mod schema_sql;
mod scoreboard;
#[cfg(test)]
mod test;
mod utils;
mod verification;
/**
 * Here is where we store all our configuration needed at runtime
 * it implements Deserialize for interfacing with figiment deserializer
*/
#[derive(Deserialize)]
pub struct InsignoConfig {
    media_folder: String,
    template_folder: String,
    intro_images: Vec<String>,
    oldest_supported_version: String,
    smtp: SmtpConfig,
    scrypt: InsignoScryptParams<'static>,
}

/**
 * Wouldn't be wonderful if we could have an easy struct for mapping trash-id to trash-names?
 */
pub struct TrashTypeMap {
    pub to_string: BTreeMap<i64, String>,
    pub to_i64: BTreeMap<String, i64>,
}

/**
is my app version compatible with the server api version? Only this http-get knows...
*/
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
    });
    if let Some(y) = result {
        Ok(y.to_string())
    } else {
        Ok(true.to_string())
    }
}

#[get("/test")]
fn test_prova() -> Result<(), InsignoError> {
    Err(InsignoError::new(401).both("wtf"))
}

/**
 here is where all the magic appens.
 calling this function we are initializing all our parameter, loading values, connecting to db...
*/
#[launch]
pub fn rocket() -> _ {
    // we need a prometheus object that implements /metric for us (and for Graphana)
    let prometheus = PrometheusMetrics::new();
    prometheus
        .registry()
        .register(Box::new(ProcessCollector::for_self()))
        .unwrap();
    /*figment is our config manager. here we define defaults parameter and how overwrite them.
    in particular:
    - default values
    - values from Insigno.toml
    - values from INSIGNO_ env variables*/
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(InsignoScryptParams::default()).key("scrypt"))
        .merge(Toml::file("Insigno.toml").nested())
        .merge(Env::prefixed("INSIGNO_").global());
    // Gimme the CONFIG
    let mut insigno_config: InsignoConfig = figment.extract().unwrap();
    insigno_config.scrypt.sem = Some(Arc::new(Semaphore::new(3)));
    // we extract database config for appending to Rocket.toml config (it's needed for rocket_sync_db_pool)
    let databases = figment.find_value("databases").unwrap();
    let secret_key = figment.find_value("secret_key").ok();
    // virtualy add DatabaseConfig to Roket.toml
    let rocket_figment = Config::figment()
        .merge(Serialized::defaults(databases).key("databases"))
        .merge(Serialized::defaults(secret_key).key("secret_key"));
    rocket::custom(rocket_figment)
        .attach(db::stage())
        .mount("/pills", pills::get_routes())
        .mount("/map", map::get_routes())
        .mount("/", auth::get_routes())
        .mount("/", routes![compatibile])
        .manage(insigno_config)
        .attach(AdHoc::on_ignite("checking config", |rocket| async {
            // if media folder does not exist it creates it
            let cfg: &InsignoConfig = rocket.state().unwrap();
            let _ = fs::create_dir_all(cfg.media_folder.clone());
            rocket
        }))
        .attach(pending::stage())
        .attach(mail::stage())
        //Cors request handler
        .attach(cors::Cors)
        .attach(verification::stage())
        .attach(scoreboard::stage())
        .attach(files::stage())
        .mount("/", cors::get_routes())
        //attach prometheus view
        .attach(prometheus.clone())
        .mount("/metrics", prometheus)
        .mount("/", routes![test_prova])
        .manage(ScryptSemaphore::default())
    //.register("/", catchers![not_found])
}
