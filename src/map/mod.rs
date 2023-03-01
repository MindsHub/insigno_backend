use std::collections::BTreeMap;

use std::error::Error;

use crate::auth::TrashTypeMap;
use crate::utils::*;
use diesel::RunQueryDsl;
use diesel::*;

use postgis::ewkb::Point;
use postgis_diesel::*;

use rocket::form::Form;
use rocket::Route;
use rocket::State;

use super::db::Db;
use super::schema_sql::*;
use rocket::response::Debug;
use rocket::serde::{json::Json, Deserialize};
use rocket_auth::User;

use crate::schema_rs::*;

use self::image::*;
mod image;


#[get("/get_near?<x>&<y>&<srid>")]
async fn get_near(
    connection: Db,
    x: f64,
    y: f64,
    srid: Option<i32>,
) -> Result<Json<Vec<Marker>>, String> {
    let tmp_point = Point {
        x,
        y,
        srid: Some(srid.unwrap_or(4326)),
    };
    let cur_point = PointC { v: tmp_point };
    connection
        .run(move |conn| {
            let t_point = st_transform(cur_point, 25832);
            let mut query = markers::table.into_boxed();
            query = query.filter(st_dwithin(
                st_transform(markers::point, 25832),
                t_point,
                15000.0,
            ));
            query.load(conn)
        })
        .await
        .map_or_else(|x| Err(x.to_string()), |x| Ok(Json(x)))
}

#[derive(Deserialize, FromForm)]
struct AddTrashField {
    x: f64,
    y: f64,
    type_tr: String,
}

#[post("/add", data = "<data>")]
async fn add_map(
    data: Form<AddTrashField>,
    user: User,
    connection: Db,
    trash_types_map: &State<TrashTypeMap>,
) -> Result<String, Debug<Box<dyn Error>>> {
    let type_int = *trash_types_map
        .to_i64
        .get(data.type_tr.to_lowercase().trim())
        .unwrap_or(&1);
    let z = Marker {
        id: None,
        created_by: user.id() as i64,
        point: PointC {
            v: Point {
                x: data.x,
                y: data.y,
                srid: Some(4326),
            },
        },
        creation_date: None,
        trash_type_id: type_int,
    };
    use markers::dsl::markers as mrkt;
    match connection
        .run(move |conn| insert_into(mrkt).values(&z).get_result::<Marker>(conn))
        .await
    {
        Ok(x) => Ok(x
            .id
            .ok_or(str_to_debug("id not found (very strange)"))?
            .to_string()),
        Err(x) => Err(Debug(x.into())),
    }
}

#[get("/types")]
async fn get_types(trash_types_map: &State<TrashTypeMap>) -> Json<BTreeMap<i64, String>> {
    Json(trash_types_map.to_string.clone())
}

pub fn get_routes() -> Vec<Route> {
    routes![get_near, get_types, add_map, add_image]
}
