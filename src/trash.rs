
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::InsignoConfig;
use crate::utils::*;
use chrono::Utc;
use diesel::RunQueryDsl;
use diesel::*;

use postgis::ewkb::Point;
use postgis_diesel::*;

use rocket::Config;
use rocket::State;
use rocket::fairing::AdHoc;
use rocket::http::ContentType;
use rocket::Data;
use rocket::Route;

use rocket_auth::User;
use rocket_multipart_form_data::mime;
use rocket_multipart_form_data::MultipartFormData;
use rocket_multipart_form_data::MultipartFormDataField;
use rocket_multipart_form_data::MultipartFormDataOptions;
use serde::ser::SerializeStruct;
use serde::Serialize;
use super::db::Db;
use rocket::serde::{Deserialize, json::Json};
use super::schema::*;
#[derive(Serialize, Clone, Queryable, Debug)]
#[diesel(table_name = "trash_types")]
struct TrashType {
    id: i32,
    name: String,
}

#[derive(Clone, Queryable, Insertable,  Debug)]
#[diesel(table_name = marker)]
struct Marker {
    #[diesel(deserialize_as = "i32")]
    id: Option<i32>,
    point: PointC<Point>,
    #[diesel(deserialize_as = "chrono::DateTime<Utc>")]
    creation_date: Option<chrono::DateTime<Utc>>,
    created_by: i32,
    trash_type_id: i32,
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
#[derive(Clone, Queryable, Insertable,  Debug)]
#[diesel(table_name = image)]
struct MarkerImage{
    #[diesel(deserialize_as = "i32")]
    id: Option<i32>,
    path: String,
    refers_to: i32,
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
//TODO add image (user, trashId, enum type)
//TODO add trash (location, user, type (enum?))

#[derive(Deserialize)]
struct AddTrashField{
    x: f64,
    y: f64,
    typeTr: String,
}

#[post("/add", format = "json", data = "<data>")]
async fn add_trash(data: Json<AddTrashField>, user: User, connection: Db)-> Option<String>{
    
    let z = Marker{
        id: None,
        created_by: user.id(),
        point: PointC { v: Point { x: data.x, y: data.y, srid: Some(4326) } },
        creation_date: None,
        trash_type_id: 1
    };
    use markers::dsl::markers as mrkt;
    if let Ok(test) = connection.run(move |conn| {
        insert_into(mrkt).values(&z).get_result::<Marker>(conn)
    }).await{
        return Some(test.id.unwrap().to_string());
    }
    None
}

#[post("/image/add", data = "<data>")]
async fn add_image(content_type: &ContentType, data: Data<'_>, user: User, connection: Db, config: &State<InsignoConfig>) -> Option<String> {
    user.id();
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("creationPhoto")
            .content_type_by_string(Some(mime::IMAGE_PNG))
            .unwrap(),
    ]);
    let mut custom = PathBuf::new();

    custom.set_file_name(&config.mediaFolder);

    let multipart_form_data = MultipartFormData::parse(content_type, data, options)
        .await
        .unwrap();
    let photo = multipart_form_data.files.get("creationPhoto");
    if let Some(tmp) = photo {
        let x =&tmp[0];
        let new_pos = unique_path(&custom, Path::new("png"));
        println!("{:?}", new_pos);
        fs::copy(&x.path, &new_pos).unwrap_or_else(|x| {println!("{}", x.to_string()); 0});
        let z = new_pos.strip_prefix(custom.to_str().unwrap()).unwrap();
        let img = MarkerImage{
            id: None,
            path: z.to_str().unwrap().to_string(),
            refers_to: 4,
        };
        if let Ok(z) = connection.run(move |conn|{
            use marker_images::dsl::marker_images as mi;
            insert_into(mi).values(&img).get_result::<MarkerImage>(conn)
        }).await{
            return Some(z.id.unwrap().to_string());
        }
    }
    None
}


#[get("/types")]
async fn get_types(connection: Db) -> Option<Json<Vec<TrashType>>> {
    let res: Result<Vec<TrashType>, _> = connection.run(|x| trash_types::table.load(x)).await;
    res.map(|x| Json(x)).ok()
}

pub fn get_routes() -> Vec<Route> {
    routes![get_near, get_types, add_trash, add_image]
}
