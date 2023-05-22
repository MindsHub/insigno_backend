use rocket::fairing::AdHoc;

use rocket_sync_db_pools::{database, diesel::PgConnection};

/// our connection pool
#[database("db")]
pub struct Db(pub PgConnection);

///init our connection
pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel PostGres Stage", |rocket| async {
        rocket.attach(Db::fairing())
    })
}
