
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use diesel::QueryDsl;
use diesel::RunQueryDsl;
use postgis_diesel::*;
use rocket::Route;
use rocket::http::ContentType;
use rocket::serde::json::Json;
use rocket_auth::User;
use rocket_multipart_form_data::MultipartFormData;
use rocket_multipart_form_data::MultipartFormDataField;
use rocket_multipart_form_data::MultipartFormDataOptions;
use rocket_multipart_form_data::mime;
use serde::Serialize;
use serde::ser::SerializeStruct;
use postgis::ewkb::Point;
use diesel::*;
use rocket::Data;
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
        creation_photo_path->Text,
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
    creation_photo_path: String,
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
            query.load(conn)
            })
        .await
        .map_or_else(|x| Err(x.to_string()), |x| Ok(Json(x)))
}

#[post("/add", data = "<data>")]
async fn add(content_type: &ContentType, data: Data<'_>, _user: User) -> Result<(), String>{
   let options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec! [
            MultipartFormDataField::file("creationPhoto").content_type_by_string(Some(mime::IMAGE_PNG)).unwrap(),
        ]
    );
    let mut custom = PathBuf::new();
    
    custom.set_file_name("/home/alessio/Documents/Mindshub/insignio/insignio_backend/test");

    let mut multipart_form_data = MultipartFormData::parse(content_type, data, options).await.unwrap();
    let photo = multipart_form_data.files.get("creationPhoto");
    if let Some(tmp) = photo{
        for x in tmp{
            let new_pos = unique_path(&custom, Path::new("png"));
            println!("{:?}", new_pos);
            fs::copy(&x.path, new_pos).map_err(|x| x.to_string())?;
            
        }
    }
    Ok(())

}

#[get("/types")]
async fn get_types(connection: Db)->Option< Json<Vec<TrashType>>> {
    let res: Result<Vec<TrashType>, _> = connection
        .run(|x| trash_types::table.load(x))
        .await;
    res.map(|x| Json(x)).ok()
}

pub fn get_routes() -> Vec<Route> {
    routes![get_near, get_types, add]
}
