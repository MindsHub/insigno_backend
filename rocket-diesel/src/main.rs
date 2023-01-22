#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;

use diesel::{prelude::*, table, Insertable, Queryable};
use rocket::{fairing::AdHoc, serde::json::Json, State};
use rocket_sync_db_pools::database;
use serde::{Deserialize, Serialize};


table! {
    pills (id) {
        id -> Int4,
        text -> Text,
        author -> Text,
        source -> Text,
    }
}

#[database("db")]
pub struct Db(diesel::PgConnection);

#[derive(Serialize, Deserialize, Clone, Queryable, Debug, Insertable)]
#[table_name = "pills"]
struct Pill {
    id: i32,
    text: String,
    author: String,
    source: String,
}


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/")]
async fn get_random_pill(connection: Db) -> Json<Option<Vec<Pill>>> {
    let res = connection
        .run(|c| pills::table.load(c))
        .await
        .map(Json);
    if let Ok(Json(res)) = res{
        Json(Some(res))
    }else{
        Json(None)
    }
        
}


#[derive(Deserialize)]
struct Config {
    name: String,
    age: u8,
}


#[get("/get_config")]
fn get_config(config: &State<Config>) -> String {
    format!(
      "Hello, {}! You are {} years old.", config.name, config.age
   )
}


#[launch]
fn rocket() -> _ {
    let rocket= rocket::build();
    rocket
    .attach(Db::fairing())
    .attach(AdHoc::config::<Config>())
    
    .mount("/pills/random", routes![get_random_pill])
    .mount("/", routes![index, get_config])
    
}
