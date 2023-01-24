use rocket::serde::json::Json;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod db;
mod pills;
mod trash;

#[get("/test")]
fn test() -> Json<String> {
    Json("ok".to_string())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(db::stage())
        .mount("/pills", pills::get_routes())
        .mount("/", routes![test])
    //.mount("/trash", trash::get_routes())
}
