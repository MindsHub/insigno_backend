use chrono::{DateTime, Utc};
use diesel::{select, RunQueryDsl};
use rocket::fairing::AdHoc;

use crate::{
    auth::user::{Adult, Authenticated, User},
    db::Db,
    utils::InsignoError,
};

use self::sql::{time_to_verify, can_verify};

mod sql;
struct ImageVerifications;
impl ImageVerifications {
    pub async fn time_to_verify(
        user_id: i64,
        db: &Db,
    ) -> Result<DateTime<Utc>, diesel::result::Error> {
        db.run(move |conn| select(time_to_verify(user_id)).get_result::<DateTime<Utc>>(conn))
            .await
    }
    pub async fn can_verify(
        user: &User<Authenticated, Adult>,
        db: &Db,
    ) -> Result<bool, diesel::result::Error> {
        let user_id=user.id.unwrap();
        db.run(move |conn| select(can_verify(user_id)).get_result(conn))
            .await
    }

    pub async fn get_or_create(user_id: i64, db: &Db) {}
}

// time remaining on token refresh (and token last 1y) /session
// dammi la sessione con la roba da verificare
// punti guadagnati/sessione finita dopo verifica

#[get("/get_session")]
pub async fn get_session(
    user: Result<User<Authenticated, Adult>, InsignoError>,
    db: Db,
) -> Result<(), InsignoError> {
    let user = user?;
    let z = ImageVerifications::can_verify(&user, &db)
        .await
        .map_err(|x| InsignoError::new(500).debug(x.to_string()))?;
    let z = Utc::now();
    todo!()
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("verification stage", |rocket| async { rocket })
}
