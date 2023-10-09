use chrono::{DateTime, Utc};
use diesel::{select, sql_query, sql_types::BigInt, RunQueryDsl};
use rocket::{fairing::AdHoc, serde::json::Json};

use crate::{
    auth::user::{Authenticated, User, YesReview},
    db::Db,
    utils::InsignoError,
};

use self::sql::{time_to_verify, ImageVerification};

mod sql;
struct ImageVerifications;
impl ImageVerifications {
    pub async fn time_to_verify(
        user_id: i64,
        db: &Db,
    ) -> Result<DateTime<Utc>, diesel::result::Error> {
        db //
            .run(move |conn| select(time_to_verify(user_id))
            .get_result::<DateTime<Utc>>(conn))
            .await
    }

    pub async fn get_or_create(
        user_id: i64,
        db: &Db,
    ) -> Result<Vec<ImageVerification>, diesel::result::Error> {
        db.run(move |conn| {
            sql_query("SELECT * FROM get_to_verify($1)")
                .bind::<BigInt, _>(user_id)
                .get_results(conn)
        })
        .await
    }
}

// time remaining on token refresh (and token last 1y) /session
// dammi la sessione con la roba da verificare
// punti guadagnati/sessione finita dopo verifica

#[get("/get_next_verify_time")]
pub async fn get_next_verify_time(
    user: Result<User<Authenticated, YesReview>, InsignoError>,
    db: Db,
) -> Result<Json<DateTime<Utc>>, InsignoError> {
    let user = user?;
    let z = ImageVerifications::time_to_verify(user.id.unwrap(), &db)
        .await
        .map_err(|x| InsignoError::new(500).debug(x))?;
    Ok(Json(z))
}

#[get("/get_session")]
pub async fn get_session(
    user: Result<User<Authenticated, YesReview>, InsignoError>,
    db: Db,
) -> Result<Json<Vec<ImageVerification>>, InsignoError> {
    let user = user?;
    let z = ImageVerifications::get_or_create(user.id.unwrap(), &db)
        .await
        .map_err(|x| {
            match x {
                diesel::result::Error::DatabaseError(x, y) => {
                    if format!("{y:?}") != "you cant verify right now" {
                        InsignoError::new(403).both(format!("{y:?}"))
                    } else {
                        InsignoError::new(500).debug(format!("{x:?}"))
                    }
                }
                _ => InsignoError::new(500).debug(x),
            }
            //InsignoError::new(500).debug(x)
        })?;
    Ok(Json(z))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("verification stage", |rocket| async {
        rocket.mount("/verify", routes![get_session, get_next_verify_time])
    })
}
