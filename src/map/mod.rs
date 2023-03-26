use std::collections::BTreeMap;

use std::error::Error;

use crate::auth::*;
use crate::utils::*;
use crate::TrashTypeMap;
use chrono::Utc;
use diesel::RunQueryDsl;
use diesel::*;

use diesel::sql_types::*;

use postgis_diesel::sql_types::Geometry;
use rocket::form::Form;
use rocket::Route;
use rocket::State;

use serde::Serialize;

use super::db::Db;
use super::schema_sql::*;
use rocket::response::Debug;
use rocket::serde::{json::Json, Deserialize};

use self::image::*;
use crate::schema_rs::*;
use rocket::http::Status;
mod image;
use postgis_diesel::types::Point;
#[get("/get_near?<x>&<y>&<srid>&<include_resolved>")]
async fn get_near(
    connection: Db,
    x: f64,
    y: f64,
    srid: Option<u32>,
    include_resolved: Option<bool>,
) -> Result<Json<Vec<Marker>>, Debug<Box<dyn Error>>> {
    /*let tmp_point = Point {
        x,
        y,
        srid: Some(srid.unwrap_or(4326)),
    };*/
    let cur_point = Point { x,
        y,
        srid: Some(srid.unwrap_or(4326u32)), };
    let res: Vec<Marker> = connection
        .run(move |conn| {
            let query = sql_query(
                "SELECT *
                FROM markers 
                WHERE ST_DWITHIN(point, $1, 0.135) 
                AND (resolution_date IS NULL OR $2) 
                AND (SELECT COUNT (*) FROM marker_reports WHERE markers.id = reported_marker)<3",
            )
            .bind::<Geometry, _>(cur_point)
            .bind::<Bool, _>(include_resolved.unwrap_or(true));

            query.get_results(conn)
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
) -> Result<Json<MarkerUpdate>, InsignoError> {
    #[derive(QueryableByName, Debug)]
    struct PointRet {
        #[sql_type = "BigInt"]
        add_marker: i64,
    }
    let type_int = if trash_types_map
        .to_string
        .contains_key(&data.marker_types_id)
    {
        data.marker_types_id
    } else {
        1
    };

    let created_id: PointRet = connection
        .run(move |conn| {
            //add_marker(user_id BIGINT, point GEOMETRY, trash_type BIGINT)
            sql_query(
                "
            SELECT * FROM add_marker($1, $2, $3);",
            )
            .bind::<BigInt, _>(user.id.unwrap())
            .bind::<Geometry, _>(Point::new(data.x, data.y, None))//(InsignoPoint::new(data.x, data.y))
            .bind::<BigInt, _>(type_int)
            .get_result(conn)
        })
        .await
        .map_err(|x| InsignoError::new(404, "id not found", &x.to_string()))?;
    Ok(Json(MarkerUpdate {
        id: created_id.add_marker,
        earned_points: 1.0,
    }))
}

#[get("/types")]
async fn get_types(trash_types_map: &State<TrashTypeMap>) -> Json<BTreeMap<i64, String>> {
    Json(trash_types_map.to_string.clone())
}

#[derive(Serialize)]
pub struct MarkerInfo {
    id: i64,
    point: InsignoPoint,
    creation_date: chrono::DateTime<Utc>,
    resolution_date: Option<chrono::DateTime<Utc>>,
    created_by: Option<UnautenticatedUser>,
    solved_by: Option<UnautenticatedUser>,
    marker_types_id: i64,
    can_report: bool,
    images_id: Option<Vec<i64>>,
}

impl From<Marker> for MarkerInfo {
    fn from(value: Marker) -> Self {
        MarkerInfo {
            id: value.id.unwrap(),
            point: value.point,
            creation_date: value.creation_date.unwrap(),
            resolution_date: value.resolution_date,
            created_by: None,
            solved_by: None,
            marker_types_id: value.marker_types_id,
            can_report: false,
            images_id: None,
        }
    }
}
#[derive(Serialize)]
struct MarkerUpdate {
    id: i64,
    earned_points: f64,
}

#[get("/<marker_id>")]
async fn get_marker_from_id(
    marker_id: i64,
    connection: Db,
    user: Option<User>,
) -> Result<Json<MarkerInfo>, InsignoError> {
    let m: Marker = connection
        .run(move |conn| markers::table.filter(markers::id.eq(marker_id)).load::<Marker>(conn))
        .await
        .map_err(|x| InsignoError::new_debug(404, &x.to_string()))?
        .get(0)
        .ok_or(InsignoError::new(
            404,
            "marker not found",
            "marker not found",
        ))?
        .clone();
    let creation_user = get_user_by_id(&connection, m.created_by)
        .await
        .map_err(|x| InsignoError::new_debug(404, &x.to_string()))?;
    let solved_by_user = if let Some(s) = m.solved_by {
        Some(
            get_user_by_id(&connection, s)
                .await
                .map_err(|x| InsignoError::new_debug(404, &x.to_string()))?
                .into(),
        )
    } else {
        None
    };
    let mut m: MarkerInfo = m.into();
    m.created_by = Some(creation_user.into());
    m.solved_by = solved_by_user;
    m.images_id = Some(_list_image(marker_id, &connection).await?);

    let v: Vec<MarkerReport> = connection
        .run(move |conn| {
            let query = marker_reports::table.filter(marker_reports::reported_marker.eq(marker_id));
            if let Some(user) = user {
                query
                    .filter(marker_reports::user_f.eq(user.id.unwrap()))
                    .get_results(conn)
            } else {
                query.get_results(conn)
            }
        })
        .await
        .map_err(|x| InsignoError::new(404, "", &x.to_string()))?;
    if v.is_empty() {
        m.can_report = true;
    }

    Ok(Json(m))
}

#[derive(QueryableByName, Debug)]
struct ResolveRet {
    #[sql_type = "Double"]
    resolve_marker: f64,
}

#[post("/resolve/<marker_id>")]
async fn resolve_marker_from_id(
    marker_id: i64,
    user: User,
    connection: Db,
) -> Result<Json<MarkerUpdate>, Status> {
    let y: ResolveRet = connection
        .run(
            move |conn| {
                sql_query(
                    "
            SELECT * FROM resolve_marker($1, $2);",
                )
                .bind::<BigInt, _>(marker_id)
                .bind::<BigInt, _>(user.id.unwrap())
                .get_result(conn)
            }, //select(resolve_marker(marker_id, user.id.unwrap())).execute(conn))
        )
        .await
        .map_err(|tmp| match tmp.to_string().as_str() {
            "marker_non_trovato" => Status::NotFound,
            "marker_risolto" => Status::BadRequest,
            _ => Status::InternalServerError,
        })?;

    Ok(Json(MarkerUpdate {
        id: marker_id,
        earned_points: y.resolve_marker,
    }))
}

#[post("/report/<marker_id>")]
async fn report_marker(marker_id: i64, user: User, connection: Db) -> Result<(), InsignoError> {
    connection
        .run(move |conn| {
            let query = sql_query(
                "INSERT INTO marker_reports(user_f, reported_marker)
                SELECT $1, $2
                WHERE NOT EXISTS (SELECT *
                        FROM marker_reports
                        WHERE user_f=$1 AND reported_marker=$2)
                returning *;",
            )
            .bind::<BigInt, _>(user.id.unwrap())
            .bind::<BigInt, _>(marker_id);

            query.get_result::<MarkerReport>(conn)
        })
        .await
        .map_err(|x| {
            InsignoError::new(
                422,
                "Impossible to report. Maybe you already reported",
                &x.to_string(),
            )
        })?;

    Ok(())
}

pub fn get_routes() -> Vec<Route> {
    routes![
        get_near,
        get_types,
        add_map,
        add_image,
        list_image,
        get_image,
        resolve_marker_from_id,
        report_marker,
        get_marker_from_id,
    ]
}

#[cfg(test)]
mod test {
    use crate::rocket;
    use crate::test::{test_add_point, test_reset_db, test_signup};
    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
        serde::json::{serde_json, Value},
    };

    #[rocket::async_test]
    async fn test_marker_report() {
        test_reset_db();
        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");

        //signup
        test_signup(&client).await;

        //test without markers
        let response = client.post("/map/report/1").dispatch().await;
        assert_eq!(response.status(), Status::UnprocessableEntity);

        //add marker
        test_add_point(&client).await;

        //test with marker
        let response = client.post("/map/report/1").dispatch().await;
        assert_eq!(response.status(), Status::Ok);

        //test report again
        let response = client.post("/map/report/1").dispatch().await;
        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

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

        //signup
        test_signup(&client).await;

        let response = client
            .post("/map/add")
            .header(ContentType::Form)
            .body("x=0.0&y=0.0&marker_types_id=2")
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().await.unwrap(),
            "{\"id\":1,\"earned_points\":1.0}"
        );

        let response = client.post("/map/resolve/1").dispatch().await;
        assert_eq!(response.status(), Status::Ok);

        let response = client.get("/map/1").dispatch().await;
        assert_eq!(response.status(), Status::Ok);

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
        test_add_point(&client).await;

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
