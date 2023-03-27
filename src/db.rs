use rocket::fairing::AdHoc;

use rocket_sync_db_pools::{database, diesel::PgConnection};

#[database("db")]
pub struct Db(pub PgConnection);

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel PostGres Stage", |rocket| async {
        rocket.attach(Db::fairing())
    })
}
