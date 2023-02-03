use diesel::QueryDsl;
use diesel::RunQueryDsl;
use postgis_diesel::*;
use rocket::Route;
use rocket::serde::json::Json;
use serde::Serialize;
use serde::ser::SerializeStruct;
use postgis::ewkb::Point;
use diesel::*;
use crate::utils::*;

use super::db::Db;

table! {
    trash_types (id){
        id->Integer,
        name->Text,
    }
}

table! {
    marker(id) {
        id -> Integer,
        point-> postgis_diesel::sql_types::Geometry,
        creation_date->Timestamptz,
        trash_type_id -> Integer,
    }
}
#[derive(Serialize, Clone, Queryable, Debug, Insertable)]
#[diesel(table_name = "trash_types")]
struct TrashType {
    id: i32,
    name: String,
}

#[derive(Clone, Queryable, Debug)]
#[diesel(table_name = marker)]
struct Marker {
    id: i32,
    point: PointC<Point>,
    creation_date: chrono::NaiveDateTime,
    trash_type_id: i32,
}

impl Serialize for Marker{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            let mut s = serializer.serialize_struct("Person", 3)?;
            s.serialize_field("id", &self.id)?;
            s.serialize_field("point", &InsignoPoint::from(self.point))?;
            s.serialize_field("creation_date", &InsignoTimeStamp::from(self.creation_date))?;
            s.serialize_field("trash_type_id", &self.trash_type_id)?;
            s.end()
    }
}


#[get("/get_near?<x>&<y>&<srid>")]
async fn get_near(connection: Db, x: f64, y: f64, srid: Option<i32>) ->Result<Json<Vec<Marker>>, String>{
    let tmp_point = Point{x, y, srid: Some(srid.unwrap_or(4326))};
    let cur_point = PointC{
        v: tmp_point
    };
    connection
        .run(move |conn| {
            let t_point = st_transform(cur_point, 25832);
            let mut query = marker::table.into_boxed();
            query = query.filter(st_dwithin(st_transform(marker::point, 25832), t_point, 15000.0));
            //let debug = diesel::debug_query::<diesel::pg::Pg, _>(&query);
            //println!("The insert query: {}", debug.to_string());
            query.load(conn)
            })
        .await
        .map_or_else(|x| Err(x.to_string()), |x| Ok(Json(x)))
}



#[get("/types")]
async fn get_types(connection: Db)->Option<Json<Vec<TrashType>>> {
    let res: Result<Vec<TrashType>, _> = connection
        .run(|x| trash_types::table.load(x))
        .await;

    if let Ok(ret) = res{
        Some(Json(ret))
    }else{
        None
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![get_near, get_types]
}
