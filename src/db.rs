use rocket::fairing::AdHoc;
use rocket_sync_db_pools::database;


#[database("db")]
pub struct Db(diesel::PgConnection);

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel PostGres Stage", |rocket| async {
        rocket.attach(Db::fairing())
    })
}