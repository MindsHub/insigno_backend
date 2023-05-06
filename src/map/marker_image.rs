use std::{fs, path::Path};

use crate::diesel::RunQueryDsl;
use crate::{db::Db, utils::InsignoError, InsignoConfig};
use diesel::{sql_query, sql_types::BigInt};
use serde::Serialize;
use serde::ser::SerializeStruct;

table! {
    marker_images(id){
        id -> BigInt,
        path -> Text,
        refers_to -> BigInt,
        approved -> Bool,
    }
}
#[derive(Clone, Queryable, Insertable, Debug, QueryableByName)]
#[diesel(table_name = marker_images)]
pub struct MarkerImage {
    #[diesel(deserialize_as = i64)]
    pub id: Option<i64>,
    pub path: String,
    pub refers_to: i64,
    pub approved: bool,
}

impl Serialize for MarkerImage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("MarkerImage", 2)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("refers_to", &self.refers_to)?;
        s.end()
    }
}

impl MarkerImage {
    pub async fn approve(connection: &Db, image_id: i64) -> Result<(), InsignoError> {
        connection
            .run(move |conn| {
                sql_query(
                    "UPDATE marker_images
            SET approved=true
            WHERE id=$1",
                )
                .bind::<BigInt, _>(image_id)
                .execute(conn)
            })
            .await
            .map_err(|e| InsignoError::new(500, "impossibile verificare", &e.to_string()))?;
        Ok(())
    }

    pub async fn delete(
        connection: &Db,
        image_id: i64,
        config: &InsignoConfig,
    ) -> Result<Self, InsignoError> {
        let img: MarkerImage = connection
            .run(move |conn| {
                sql_query(
                    "DELETE
            FROM marker_images
            WHERE id=$1
            RETURNING *",
                )
                .bind::<BigInt, _>(image_id)
                .get_result(conn)
            })
            .await
            .map_err(|e| {
                InsignoError::new(
                    500,
                    "impossibile cancellare, id non trovato",
                    &e.to_string(),
                )
            })?;
        let img_path = Path::new(&config.media_folder).join(&img.path);
        let _ = fs::remove_file(img_path);
        Ok(img)
    }

    pub async fn get_to_report(connection: &Db) -> Result<Vec<Self>, InsignoError> {
        let res: Vec<Self> = connection
            .run(|conn| {
                sql_query(
                    "SELECT marker_images.*
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
