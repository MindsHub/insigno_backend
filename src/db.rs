use std::{collections::BTreeMap, error::Error};

use diesel::{pg::Pg, RunQueryDsl};
use diesel_migrations::{self, embed_migrations, EmbeddedMigrations, MigrationHarness};
use rocket::{fairing::AdHoc, Build, Rocket};

use rocket_sync_db_pools::{database, diesel::PgConnection};

use crate::{schema_sql::marker_types, TrashTypeMap};

/// our connection pool
#[database("db")]
pub struct Db(pub PgConnection);

///init our connection
pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel PostGres Stage", |rocket| async {
        rocket
            .attach(Db::fairing())
            .attach(AdHoc::try_on_ignite("Migrations", run_db_migrations))
    })
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

async fn run_db_migrations(rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
    let conn = Db::get_one(&rocket).await.expect("database connection");
    let rocket = conn
        .run(
            |db: &mut PgConnection| match db.run_pending_migrations(MIGRATIONS) {
                Ok(_) => Ok(rocket),
                Err(e) => {
                    error!("Failed to run database migrations: {:?}", e);
                    Err(rocket)
                }
            },
        )
        .await?;
    let t = conn
        .run(|conn: &mut PgConnection| {
            let sorted = marker_types::table
                .load::<(i64, String, f64)>(conn)
                .unwrap()
                .into_iter()
                .map(|(x, y, ..)| (x, y))
                .collect::<BTreeMap<i64, String>>();
            let inverted = sorted.clone().into_iter().map(|(x, y)| (y, x)).collect();
            TrashTypeMap {
                to_string: sorted,
                to_i64: inverted,
            }
        })
        .await;
    let rocket = rocket.manage(t);
    Ok(rocket)
}

pub fn run_migrations(connection: &mut impl MigrationHarness<Pg>) -> Result<(), Box<dyn Error>> {
    if let Ok(x) = connection.pending_migrations(MIGRATIONS) {
        if !x.is_empty() {
            let s = x
                .into_iter()
                .map(|x| x.name().to_string())
                .fold(String::new(), |x, y| x + "\n\t" + &y);
            info!("Running migrations: {}", s)
        }
    }
    connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(|_| "unknown")?;
    Ok(())
}
