use std::collections::BTreeMap;

use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::auth::TrashTypeMap;
use crate::utils::*;
use crate::InsignoConfig;
use chrono::Utc;
use diesel::RunQueryDsl;
use diesel::*;

use postgis::ewkb::Point;
use postgis_diesel::*;

use rocket::form::Form;
use rocket::http::ContentType;
use rocket::Data;
use rocket::Route;
use rocket::State;

use super::db::Db;
use super::schema::*;
use rocket::serde::{json::Json, Deserialize};
use rocket_auth::User;
use rocket_multipart_form_data::mime;
use rocket_multipart_form_data::MultipartFormData;
use rocket_multipart_form_data::MultipartFormDataField;
use rocket_multipart_form_data::MultipartFormDataOptions;
use serde::ser::SerializeStruct;
use serde::Serialize;
use rocket::response::Debug;
#[derive(Serialize, Clone, Queryable, Debug)]
#[diesel(table_name = "trash_types")]
struct TrashType {
    id: i64,
    name: String,
}

#[derive(Clone, Queryable, Insertable, Debug)]
#[diesel(table_name = marker)]
struct Marker {
    #[diesel(deserialize_as = "i64")]
    id: Option<i64>,
    point: PointC<Point>,
    #[diesel(deserialize_as = "chrono::DateTime<Utc>")]
    creation_date: Option<chrono::DateTime<Utc>>,
    created_by: i64,
    trash_type_id: i64,
}

impl Serialize for Marker {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Marker", 4)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("point", &InsignoPoint::from(self.point))?;
        s.serialize_field("creation_date", &self.creation_date)?;
        s.serialize_field("trash_type_id", &self.trash_type_id)?;
        s.end()
    }
}
#[derive(Clone, Queryable, Insertable, Debug)]
#[diesel(table_name = image)]
struct MarkerImage {
    #[diesel(deserialize_as = "i64")]
    id: Option<i64>,
    path: String,
    refers_to: i64,
}

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
async fn add_trash(
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
        .await{
            Ok(x) =>{Ok(x.id.ok_or(Debug("id not found (very strange)".into()))?.to_string())},
            Err(x) => {Err(Debug(x.into()))}
        }
}

#[post("/image/add", data = "<data>")]
async fn add_image(
    content_type: &ContentType,
    data: Data<'_>,
    user: User,
    connection: Db,
    config: &State<InsignoConfig>,
) -> Result<(), Debug<Box<dyn Error>>> {
    // parse multipart data
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("image")
            .content_type_by_string(Some(mime::IMAGE_PNG))
            .map_err(|x| to_debug(x))?
            ,
        MultipartFormDataField::text("refers_to_id"),
    ]);

    let multipart_form_data = MultipartFormData::parse(content_type, data, options)
        .await
        .map_err(|x| to_debug(x))?;

    // cast data to normal values
    let photo_path = &multipart_form_data.files.get("image").ok_or(str_to_debug("photo field not found"))?[0];
    let id = multipart_form_data.texts.get("refers_to_id").ok_or(str_to_debug("id field not found"))?[0]
        .text
        .parse::<i64>()
        .map_err(to_debug)?;
    let user_id = user.id as i64;

    // check if user own the marker
    connection
        .run(move |conn| {
            markers::table
                .find(id)
                .filter(markers::created_by.eq(user_id))
                .load::<Marker>(conn)
        })
        .await
        .map_err(|x |to_debug(x))?;

    // find a place to save the image in memory
    let mut custom_path = PathBuf::new();
    custom_path.set_file_name(&config.media_folder);
    let new_pos = unique_path(&custom_path, Path::new("png"));
    fs::copy(&photo_path.path, &new_pos)
        .map_err(to_debug)?;
    let z = new_pos.strip_prefix(custom_path.to_str().ok_or(str_to_debug("to str doesn't work"))?)
    .map_err(to_debug)?;

    // try to save it in database
    let img = MarkerImage {
        id: None,
        path: z.to_str().ok_or(str_to_debug("to str doesn't work"))?.to_string(),
        refers_to: id,
    };
    connection
        .run(move |conn| {
            use marker_images::dsl::marker_images as mi;
            insert_into(mi).values(&img).get_result::<MarkerImage>(conn)
        })
        .await
        .map_err(|x| {
            _ = fs::remove_file(new_pos);
            to_debug(x)
        })?;

    Ok(())
}

#[get("/types")]
async fn get_types(trash_types_map: &State<TrashTypeMap>) -> Json<BTreeMap<i64, String>> {
    Json(trash_types_map.to_string.clone())
}

pub fn get_routes() -> Vec<Route> {
    routes![get_near, get_types, add_trash, add_image]
}
