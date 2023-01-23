#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;


mod pills;
mod db;
//mod trash;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(db::stage())
        .mount("/pills", pills::get_routes())
        //.mount("/trash", trash::get_routes())
}
