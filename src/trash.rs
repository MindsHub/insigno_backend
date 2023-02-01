use std::error::Error;

use diesel::RunQueryDsl;
use diesel::data_types::PgTimestamp;
use diesel::result;
use diesel::sql_types::Serial;
use postgis_diesel::operators::*;
use postgis_diesel::types::*;
use rocket::Either;
use rocket::Route;
use rocket::serde::json::Json;
use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::insigno_point::InsignoPoint;
use crate::insigno_point::InsignoTimeStamp;

use super::db::Db;

table! {
    trash_type{
        id->Integer,
        name->Text,
    }
}

table! {
    marker {
        id -> Integer,
        point-> postgis_diesel::sql_types::Geometry,
        creation_date->Timestamptz,
    }
}

#[derive(Serialize, Clone, Queryable, Debug, Insertable)]
#[diesel(table_name = trash_type)]
struct TrashType {
    id: i32,
    name: String,
}

#[derive(Clone, Queryable, Debug)]
#[diesel(table_name = marker)]
#[diesel(belongs_to(TrashType))]
struct Marker {
    id: i32,
    point: Point,
    creation_date: chrono::NaiveDateTime,
}

impl Serialize for Marker{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            let mut s = serializer.serialize_struct("Person", 3)?;
            s.serialize_field("id", &self.id)?;
            s.serialize_field("point", &InsignoPoint::from(self.point))?;
            s.serialize_field("creation_date", &InsignoTimeStamp::from(self.creation_date))?;
            s.end()
    }
}


#[get("/get_near")]
async fn get_near(connection: Db) ->Json<Result<Vec<Marker>, String>>{
    connection
        .run(|x| marker::table.load(x))
        .await
        .map_or_else(|x| Json(Err(x.to_string())), |x| Json(Ok(x)))
}

#[get("/types")]
async fn get_types(connection: Db)->Json<Option<Vec<TrashType>>> {
    let res: Result<Vec<TrashType>, _> = connection
        .run(|x| trash_type::table.load(x))
        .await;

    if let Ok(ret) = res{
        Json(Some(ret))
    }else{
        Json(None)
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![get_near, get_types]
}
