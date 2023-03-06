use std::collections::BTreeMap;

use std::error::Error;

use crate::auth::TrashTypeMap;
use crate::utils::*;
use diesel::RunQueryDsl;
use diesel::*;

use diesel::sql_types::BigInt;
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
) -> Result<Json<Vec<Marker>>, Debug<Box<dyn Error>>> {
    let tmp_point = Point {
        x,
        y,
        srid: Some(srid.unwrap_or(4326)),
    };
    let cur_point = PointC { v: tmp_point };
    let res: Vec<Marker> = connection
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
        .map_err(to_debug)?;
    Ok(Json(res))
}

#[derive(Deserialize, FromForm)]
struct AddTrashField {
    x: f64,
    y: f64,
    marker_types_id: i64,
}

#[post("/add", data = "<data>")]
async fn add_map(
    data: Form<AddTrashField>,
    user: User,
    connection: Db,
    trash_types_map: &State<TrashTypeMap>,
) -> Result<String, Debug<Box<dyn Error>>> {
    let type_int = if trash_types_map
        .to_string
        .contains_key(&data.marker_types_id)
    {
        data.marker_types_id
    } else {
        1
    };

    let z = Marker {
        id: None,
        created_by: user.id() as i64,
        solved_by: None,
        point: PointC {
            v: Point {
                x: data.x,
                y: data.y,
                srid: Some(4326),
            },
        },
        creation_date: None,
        resolution_date: None,
        marker_types_id: type_int,
    };
    use markers::dsl::markers as mrkt;
    let y = connection
        .run(move |conn| insert_into(mrkt).values(&z).get_result::<Marker>(conn))
        .await;
    println!("{:?}", y);

    match y {
        Ok(x) => {
            println!("fanculo");
            Ok(x.id
                .ok_or(str_to_debug("id not found (very strange)"))?
                .to_string())
        }
        //todo!()),
        Err(x) => {
            println!("stronzo");
            Err(to_debug(x))
        }
    }
}

#[get("/types")]
async fn get_types(trash_types_map: &State<TrashTypeMap>) -> Json<BTreeMap<i64, String>> {
    Json(trash_types_map.to_string.clone())
}


#[get("/<marker_id>")]
async fn get_marker_from_id(
    marker_id: i64,
    connection: Db,
) -> Result<Json<Marker>, Debug<Box<dyn Error>>> {
    let m: Marker = connection
        .run(move |conn| markers::table.find(marker_id).load::<Marker>(conn))
        .await
        .map_err(to_debug)?
        .get(0)
        .ok_or(str_to_debug("not found"))?
        .clone();
    Ok(Json(m))
}

sql_function!(fn resolve_marker(marker_id: BigInt, user_id: BigInt));

#[post("/resolve/<marker_id>")]
async fn resolve_marker_from_id(
    marker_id: i64,
    user: User,
    connection: Db,
) -> Result<(), Debug<Box<dyn Error>>>{
    connection
        .run(move |conn|{
            select(resolve_marker(marker_id as i64, user.id as i64)).execute(conn)
        }).await
        .map_err(to_debug)?;
    //let query = select(resolve_marker(marker_id as i64, user.id as i64));
    //let y = debug_query::<Pg, _>(&query);
    //println!("{y} {:?}", ret);
    Ok(())
}

pub fn get_routes() -> Vec<Route> {
    routes![
        get_near,
        get_types,
        add_map,
        add_image,
        get_marker_from_id,
        list_image,
        get_image,
        resolve_marker_from_id
    ]
}
