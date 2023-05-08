use crate::diesel::RunQueryDsl;
use diesel::{sql_query, sql_types::BigInt};
use serde::Serialize;

use crate::{db::Db, utils::InsignoError};

table! {
    marker_reports(id){
        id -> Nullable<BigInt>,
        user_f -> BigInt,
        reported_marker -> BigInt,
    }
}

#[derive(Clone, Queryable, Insertable, Debug, QueryableByName)]
#[diesel(table_name = marker_reports)]
pub struct MarkerReport {
    pub id: Option<i64>,
    pub user_f: i64,
    pub reported_marker: i64,
}

impl MarkerReport {
    pub async fn report(connection: &Db, user_id: i64, marker_id: i64) -> Result<(), InsignoError> {
        connection
            .run(move |conn| {
                sql_query(
                    "INSERT INTO marker_reports(user_f, reported_marker)
                        SELECT $1, $2
                        WHERE NOT EXISTS (SELECT *
                            FROM marker_reports
                            WHERE user_f=$1 AND reported_marker=$2);",
                )
                .bind::<BigInt, _>(user_id)
                .bind::<BigInt, _>(marker_id)
                .execute(conn)
            })
            .await
            .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
        Ok(())
    }
}


#[derive(Clone, QueryableByName, Serialize)]
pub struct ImageToReport{
    #[diesel(sql_type=BigInt)]
    pub id: i64,
    #[diesel(sql_type=BigInt)]
    pub marker_types_id: i64,
}
impl ImageToReport{
    pub async fn get_to_report(connection: &Db) -> Result<Vec<Self>, InsignoError> {
        let res: Vec<Self> = connection
            .run(|conn| {
                sql_query(
                    "SELECT marker_images.id, marker_types_id
                    FROM marker_images, markers
                    WHERE approved=false
                        AND markers.id=refers_to
                    ORDER BY markers.creation_date ASC
                    LIMIT 10",
                )
                .get_results(conn)
            })
            .await
            .map_err(|e| InsignoError::new_debug(500, &e.to_string()))?;
        Ok(res)
    }
}