use crate::{auth::user::users, db::Db, utils::InsignoError};
use chrono::{Utc, Timelike, Duration};
use diesel::{
    sql_query,
    sql_types::{BigInt, Float8, Text, Timestamptz},
    ExpressionMethods, NullableExpressionMethods, QueryDsl, RunQueryDsl,
};
use postgis_diesel::{sql_types::Geometry, types::Point};
use rocket::{fairing::AdHoc, serde::json::Json};
use serde::Serialize;

#[derive(Queryable, QueryableByName, Serialize)]
pub struct ScoreboardUser {
    #[diesel(sql_type=BigInt)]
    pub id: i64,
    #[diesel(sql_type=Text)]
    pub name: String,
    #[diesel(sql_type=Float8)]
    pub points: f64,
}

#[get("/global")]
pub async fn get_global_scoreboard(db: Db) -> Result<Json<Vec<ScoreboardUser>>, InsignoError> {
    let users = db
        .run(move |conn| {
            users::table
                .order(users::points.desc())
                .limit(100) // TODO implement loading more data
                .select((users::id.assume_not_null(), users::name, users::points))
                .load::<ScoreboardUser>(conn)
        })
        .await
        .map_err(|x| InsignoError::new(500).debug(x))?;
    Ok(Json(users))
}


#[derive(Serialize)]
pub struct SpecialScoreboard {
    name: Option<String>,
    users: Vec<ScoreboardUser>,
}

#[get("/special")]
pub async fn get_special_scoreboard(db: Db) -> Result<Json<SpecialScoreboard>, InsignoError> {
    // - special endpoint for the Insigno contest held at the Maker Faire Rome
    // - this can be reused for special scoreboards in general
    // - if there is no active scoreboard, just replace the function body with
    //   Ok(Json(SpecialScoreboard { name: None, vec![] }))

    let cur_point = Point {
        x: 12.311324,
        y: 41.806265,
        srid: Some(4326u32),
    };
    let radius = 31000.0; // 31km
    let min_date = Utc::now()
        .with_nanosecond(0).ok_or(InsignoError::new(500).debug("Could not set nanosecond on date"))?
        .with_second(0).ok_or(InsignoError::new(500).debug("Could not set second on date"))?
        .with_minute(0).ok_or(InsignoError::new(500).debug("Could not set minute on date"))?
        .with_hour(0).ok_or(InsignoError::new(500).debug("Could not set hour on date"))?
        - Duration::hours(2); // midnight in the italian timezone
    // println!("min_date {:?}", min_date);

    let users = db
        .run(move |conn| {
            // TODO this query hardcodes marker reporting and resolving points
            // TODO implement loading more data
            sql_query(
                "
            SELECT users.id, users.name, CAST(SUM(tbl.points) AS DOUBLE PRECISION) AS points
            FROM (
                SELECT created_by AS user_id, 1 AS points
                FROM markers
                WHERE ST_DWITHIN(point, $1, $2, FALSE)
                    AND creation_date IS NOT NULL
                    AND creation_date >= $3

                UNION ALL

                SELECT solved_by AS user_id, 10 AS points
                FROM markers
                WHERE ST_DWITHIN(point, $1, $2, FALSE)
                    AND solved_by IS NOT NULL
                    AND resolution_date IS NOT NULL
                    AND resolution_date >= $3
            ) AS tbl
            JOIN users ON tbl.user_id = users.id
            GROUP BY users.id
            ORDER BY SUM(tbl.points) DESC
            LIMIT 100
            ",
            )
            .bind::<Geometry, _>(cur_point)
            .bind::<Float8, _>(radius)
            .bind::<Timestamptz, _>(min_date)
            .get_results::<ScoreboardUser>(conn)
        })
        .await
        .map_err(|x| InsignoError::new(500).debug(x))?;

    Ok(Json(SpecialScoreboard { name: Some("Maker Faire Rome".to_string()), users }))
}

#[get("/geographical?<x>&<y>&<srid>&<radius>")]
pub async fn get_geographical_scoreboard(
    db: Db,
    x: f64,
    y: f64,
    srid: Option<u32>,
    radius: f64,
) -> Result<Json<Vec<ScoreboardUser>>, InsignoError> {
    let cur_point = Point {
        x,
        y,
        srid: Some(srid.unwrap_or(4326u32)),
    };
    let users = db
        .run(move |conn| {
            // TODO this query hardcodes marker reporting and resolving points
            // TODO implement loading more data
            sql_query(
                "
            SELECT users.id, users.name, CAST(SUM(tbl.points) AS DOUBLE PRECISION) AS points
            FROM (
                SELECT created_by AS user_id, 1 AS points
                FROM markers
                WHERE ST_DWITHIN(point, $1, $2, FALSE)

                UNION ALL

                SELECT solved_by AS user_id, 10 AS points
                FROM markers
                WHERE ST_DWITHIN(point, $1, $2, FALSE)
                    AND solved_by IS NOT NULL
            ) AS tbl
            JOIN users ON tbl.user_id = users.id
            GROUP BY users.id
            ORDER BY SUM(tbl.points) DESC
            LIMIT 100
            ",
            )
            .bind::<Geometry, _>(cur_point)
            .bind::<Float8, _>(radius)
            .get_results::<ScoreboardUser>(conn)
        })
        .await
        .map_err(|x| InsignoError::new(500).debug(x))?;
    Ok(Json(users))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("verification stage", |rocket| async {
        rocket.mount(
            "/scoreboard",
            routes![get_global_scoreboard, get_geographical_scoreboard, get_special_scoreboard],
        )
    })
}
