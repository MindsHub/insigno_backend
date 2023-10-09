pub mod sql;

use chrono::{DateTime, Utc};
use diesel::{select, sql_query, sql_types::{BigInt, Array, Nullable, Bool}, RunQueryDsl};
use rocket::{fairing::AdHoc, serde::json::Json, form::Form};
use serde::{Serialize, Deserialize};

use crate::{
    auth::user::{Authenticated, User, YesReview},
    db::Db,
    utils::InsignoError,
};

use self::sql::{time_to_verify, verify_set_verdict};


#[derive(Clone, Debug, Serialize, QueryableByName)]
pub struct ImageVerification {
    #[diesel(sql_type = BigInt)]
    image_id: i64,
    #[diesel(sql_type = BigInt)]
    marker_id: i64,
    #[diesel(sql_type = Nullable<Bool>)]
    verdict: Option<bool>,
    #[diesel(sql_type = BigInt)]
    marker_types_id: i64,
    #[diesel(sql_type = Array<BigInt>)]
    all_marker_images: Vec<i64>,
}

impl ImageVerification {
    async fn time_to_verify(
        db: &Db,
        user_id: i64,
    ) -> Result<DateTime<Utc>, diesel::result::Error> {
        db //
            .run(move |conn| select(time_to_verify(user_id))
            .get_result::<DateTime<Utc>>(conn))
            .await
    }

    async fn get_or_create(
        db: &Db,
        user_id: i64,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        db.run(move |conn| {
            sql_query("SELECT * FROM get_to_verify($1)")
                .bind::<BigInt, _>(user_id)
                .get_results(conn)
        })
        .await
    }

    async fn set_verdict(
        db: &Db,
        user_id: i64,
        image_id: i64,
        verdict: bool
    ) -> Result<bool, diesel::result::Error> {
        db //
            .run(move |conn| select(verify_set_verdict(user_id, image_id, verdict))
            .get_result::<bool>(conn))
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
    let z = ImageVerification::time_to_verify(&db, user.id.unwrap())
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
    let z = ImageVerification::get_or_create(&db, user.id.unwrap())
        .await
        .map_err(|err| match err.to_string().as_str() {
            "cant_verify_now" => InsignoError::new(403).both("You can't verify right now"),
            error_string => InsignoError::new(500).debug(error_string),
        })?;
    Ok(Json(z))
}

#[derive(Deserialize, Serialize, FromForm)]
pub struct SetVerdictData {
    image_id: i64,
    verdict: bool,
}

#[post("/set_verdict", data = "<data>")]
pub async fn set_verdict(
    user: Result<User<Authenticated, YesReview>, InsignoError>,
    db: Db,
    data: Form<SetVerdictData>
) -> Result<Json<bool>, InsignoError> {
    let user = user?;
    let z = ImageVerification::set_verdict(&db, user.id.unwrap(), data.image_id, data.verdict)
        .await
        .map_err(|err| match err.to_string().as_str() {
            "cant_verify_now" => InsignoError::new(403).both("You can't verify right now"),
            "session_not_found" => InsignoError::new(404).both("No active session with the image and the user provided"),
            error_string => InsignoError::new(500).debug(error_string),
        })?;
    Ok(Json(z))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("verification stage", |rocket| async {
        rocket.mount("/verify", routes![get_session, get_next_verify_time, set_verdict])
    })
}
