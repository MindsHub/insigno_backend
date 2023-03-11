use std::collections::BTreeMap;

use std::error::Error;

use crate::TrashTypeMap;
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

use self::image::*;
use crate::schema_rs::*;
use rocket::http::Status;
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
            let mut query = markers::table.into_boxed();
            query = query.filter(st_dwithin(
                markers::point,
                cur_point,
                0.135, // 15km/(6371 km *2pi)*360= 0.135 raggio di 15 km
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
        created_by: user.id.unwrap(),
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

    match y {
        Ok(x) => Ok(x
            .id
            .ok_or(str_to_debug("id not found (very strange)"))?
            .to_string()),
        Err(x) => Err(to_debug(x)),
    }
}

#[get("/types")]
async fn get_types(trash_types_map: &State<TrashTypeMap>) -> Json<BTreeMap<i64, String>> {
    Json(trash_types_map.to_string.clone())
}

#[get("/<marker_id>")]
async fn get_marker_from_id(marker_id: i64, connection: Db) -> Option<Json<Marker>> {
    let m: Marker = connection
        .run(move |conn| markers::table.find(marker_id).load::<Marker>(conn))
        .await
        .ok()?
        .get(0)?
        .clone();
    Some(Json(m))
}

sql_function!(fn resolve_marker(marker_id: BigInt, user_id: BigInt));

#[post("/resolve/<marker_id>")]
async fn resolve_marker_from_id(marker_id: i64, user: User, connection: Db) -> Status {
    let y = connection
        .run(move |conn| select(resolve_marker(marker_id, user.id.unwrap())).execute(conn))
        .await;
    if let Err(tmp) = y {
        match tmp.to_string().as_str() {
            "marker_non_trovato" => Status::NotFound,
            "marker_risolto" => Status::BadRequest,
            _ => Status::InternalServerError,
        }
    } else {
        Status::Ok
    }
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

#[cfg(test)]
mod test {
    use crate::rocket;
    use crate::test::{test_reset_db, test_signup};
    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
        serde::json::{serde_json, Value},
    };

    #[rocket::async_test]
    async fn test_marker_get_types() {
        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");
        // try to get types list
        let response = client.get("/map/types").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().await.unwrap(),
            r#"{"1":"unknown","2":"plastic","3":"paper","4":"undifferentiated","5":"glass","6":"compost","7":"electronics"}"#
        );
    }

    #[rocket::async_test]
    async fn test_marker_get() {
        test_reset_db();
        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");

        //get inexistent file
        let response = client.get("/map/1").dispatch().await;
        assert_eq!(response.status(), Status::NotFound);
        //assert_eq!(response.into_string().await.unwrap() , "[]");
    }

    #[rocket::async_test]
    async fn test_marker_get_near() {
        test_reset_db();

        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");

        // try to get a malformed query
        let response = client.get("/map/get_near").dispatch();
        assert_eq!(response.await.status(), Status::NotFound);

        //empty query
        let response = client.get("/map/get_near?x=0.0&y=0.0").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().await.unwrap(), "[]");

        //signup
        test_signup(&client).await;

        //add point
        let response = client
            .post("/map/add")
            .header(ContentType::Form)
            .body("x=0.0&y=0.0&marker_types_id=2")
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().await.unwrap(), "1");

        //1 point query
        let response = client.get("/map/get_near?x=0.0&y=0.0").dispatch().await;
        assert_eq!(response.status(), Status::Ok);

        //println!("{}", response.into_string().await.unwrap());
        let js: Value =
            serde_json::from_str(&response.into_string().await.unwrap()).expect("not a json");
        let arr = js.as_array().expect("not an array");
        let first = arr
            .get(0)
            .expect("empty array")
            .as_object()
            .expect("not a valid object");
        assert_eq!(first.get("id").unwrap(), 1);
        assert_eq!(first.get("resolution_date").unwrap(), &Value::Null);
        assert_eq!(first.get("created_by").unwrap(), 1);
        assert_eq!(first.get("marker_types_id").unwrap(), 2);
        let point = first.get("point").unwrap();
        assert_eq!(point.get("x").unwrap(), 0.0);
        assert_eq!(point.get("y").unwrap(), 0.0);
        assert_eq!(point.get("srid").unwrap(), 4326);

        let response = client.get("/map/get_near?x=0.136&y=0.0").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().await.unwrap(), "[]");
    }
}
/*Se un utente si iscrive a un gruppo, i marker che ha raccolto nel periodo di attività del gruppo vengono sommati ai punti del gruppo? (per me si)
oppure la relazione utente-gruppo o gruppo-gruppo ha una data di inizio e una di fine? */
