use std::{collections::BTreeMap, fs};

use auth::scrypt::InsignoScryptParams;
use diesel::{Connection, PgConnection, RunQueryDsl};
use mail::SmtpConfig;
use rocket::config::Config;
use rocket::{
    fairing::*,
    figment::{
        providers::{Env, Format, Serialized, Toml},
        Figment,
    },
    serde::Deserialize,
    State,
};
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
mod pending;

#[derive(Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct InsignoConfig {
    media_folder: String,
    template_folder: String,
    oldest_supported_version: String,
    smtp: SmtpConfig,
    scrypt: InsignoScryptParams,
}
pub struct TrashTypeMap {
    pub to_string: BTreeMap<i64, String>,
    pub to_i64: BTreeMap<String, i64>,
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Insigno config", |rocket| async {
        //generate trash_types_map
        //let value = rocket.figment().find_value("databases.db.url").unwrap();
        //let url = value.as_str().unwrap();
        let config = rocket_sync_db_pools::Config::from("db", &rocket).unwrap();

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
    });
    if let Some(y) = result {
        Ok(y.to_string())
    } else {
        Ok(true.to_string())
    }
}

#[launch]
fn rocket() -> _ {
    use rocket_prometheus::PrometheusMetrics;

    let prometheus = PrometheusMetrics::new();
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(InsignoScryptParams::default()).key("scrypt"))
        //.merge(Toml::file("Rocket.toml").nested())
        .merge(Toml::file("Insigno.toml").nested())
        .merge(Env::prefixed("INSIGNO_").global());

    let insigno_config: InsignoConfig = figment.extract().unwrap();
    let databases = figment.find_value("databases").unwrap();
    //.select(Profile::from_env_or("INSIGNO_PROFILE", "default"));

    /*rocket::custom(figment)
        .mount("/", routes![/* .. */])
        .attach(AdHoc::config::<Config>())
    };*/
    println!("{:?}", databases);
    let rocket_figment = Config::figment().merge(Serialized::defaults(databases).key("databases"));
    rocket::custom(rocket_figment)
        .attach(db::stage())
        .mount("/pills", pills::get_routes())
        .mount("/map", map::get_routes())
        .mount("/", auth::get_routes())
        .mount("/", routes![compatibile])
        .manage(insigno_config)
        //.attach(AdHoc::config::<InsignoConfig>())
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
        .attach(prometheus.clone())
        .mount("/metrics", prometheus)
    //.manage(users)
}
/*
#[get("/prova")]
async fn prova(config: &State<InsignoConfig>, db: Db)->Result<(), InsignoError>{
    let mut u = SignupInfo{ name: "Alezen".to_string(), email: "insigno@mindshub.it".to_string(), password: "PippoBaudo1!".to_string()};
    u.check(&db).await?;
    let pending = PendingUser::new(u.into_inner(), &db).await?;

    //send registration mail and insert it in db
    pending.register_and_mail(&db, mail_cfg).await?;
    Ok(())
}   */
