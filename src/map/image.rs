use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;

use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;
use diesel::insert_into;
use diesel::QueryDsl;
use rocket::response::Debug;
use rocket::Data;
use rocket::{http::ContentType, State};
use rocket_auth::User;
use rocket_multipart_form_data::*;


use crate::db::*;
use crate::schema_rs::*;
use crate::schema_sql::*;
use crate::utils::*;
use crate::InsignoConfig;

fn convert(){

}

#[post("/image/add", data = "<data>")]
pub(crate) async fn add_image(
    content_type: &ContentType,
    data: Data<'_>,
    user: User,
    connection: Db,
    config: &State<InsignoConfig>,
) -> Result<(), Debug<Box<dyn Error>>> {
    // parse multipart data
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("image")
            .content_type_by_string(Some(mime::IMAGE_STAR))
            .map_err(to_debug)?,
        MultipartFormDataField::text("refers_to_id"),
    ]);

    let mut multipart_form_data = MultipartFormData::parse(content_type, data, options)
        .await
        .map_err(to_debug)?;

    // cast data to normal values
    let photo_path = multipart_form_data
        .files
        .remove("image")
        .ok_or(str_to_debug("image field not found"))?.remove(0);

    let id = multipart_form_data
        .texts
        .get("refers_to_id")
        .ok_or(str_to_debug("id field not found"))?[0]
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
        .map_err(to_debug)?;
    
    //rename file with correct extension
    let suffix = photo_path.content_type.as_ref().ok_or(str_to_debug("no photo content type"))?.subtype().to_string().to_ascii_lowercase();
    let mut suffixed_photo_path = PathBuf::new();
    suffixed_photo_path.set_file_name(photo_path.path.to_str().ok_or(str_to_debug("no photo path available"))?);
    suffixed_photo_path.set_extension(suffix);
    fs::rename(&photo_path.path,  &suffixed_photo_path).map_err(to_debug)?;

    // find a place to save the image in memory
    let new_pos = unique_path(Path::new(&config.media_folder), Path::new("jpg"));
    process::Command::new("ffmpeg")
        .args(["-i", suffixed_photo_path.to_str().ok_or(str_to_debug("invalid path"))?, new_pos.to_str().ok_or(str_to_debug("invalid path"))?])
        .output().map_err(to_debug)?;
    //remove temporary file
    fs::remove_file(suffixed_photo_path).map_err(to_debug)?;
    
    let name = new_pos
        .strip_prefix(&config.media_folder)
        .map_err(to_debug)?;

    // try to save it in database
    let img = MarkerImage {
        id: None,
        path: name
            .to_str()
            .ok_or(str_to_debug("to str doesn't work"))?
            .to_string(),
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

#[get("/image/get")]
fn get_image(){

}