use rocket::serde::json::Json;
use serde::{Serialize, Deserialize};

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Serialize, Deserialize)]
struct Pill {
    text: String,
    author: String,
    source: String,
}

#[get("/")]
fn get_random_pill() -> Json<Pill> {
    Json(
        Pill {
            text: "test".to_string(),
            author: "test2".to_string(),
            source: "google.com".to_string(),
        }
    )
}

#[launch]
fn rocket() -> _ {
    let rocket= rocket::build();
    
    rocket
    .mount("/", routes![index])
    .mount("/pills/random", routes![get_random_pill])
}