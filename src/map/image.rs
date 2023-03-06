use std::error::Error;
use std::fs;

use std::path::Path;
use std::path::PathBuf;
use std::process;

use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;
use diesel::insert_into;
use diesel::QueryDsl;
use rocket::fs::NamedFile;
use rocket::futures::TryFutureExt;
use rocket::response::Debug;
use rocket::serde::json::Json;
use rocket::Data;
use rocket::{http::ContentType, State};
use rocket_auth::User;
use rocket_multipart_form_data::*;

use crate::db::*;
use crate::schema_rs::*;
use crate::schema_sql::*;
use crate::utils::*;
use crate::InsignoConfig;

fn convert_image(input: &PathBuf, output: &PathBuf) -> Result<(), Debug<Box<dyn Error>>> {
    println!("{input:?} {output:?}");
    if input.exists() {
        process::Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().ok_or(str_to_debug("invalid path"))?,
                output.to_str().ok_or(str_to_debug("invalid path"))?,
            ])
            .output()
            .map_err(to_debug)?;
        Ok(())
    } else {
        Err(str_to_debug("wtf bro. This is not a valid file path"))
    }
}

async fn save_image(connection: Db, name: String, id: i64) -> Result<(), Debug<Box<dyn Error>>> {
    let img = MarkerImage {
        id: None,
        path: name,
        refers_to: id,
    };

    connection
        .run(move |conn| {
            use marker_images::dsl::marker_images as mi;
            insert_into(mi).values(&img).get_result::<MarkerImage>(conn)
        })
        .await
        .map_err(to_debug)?;

    Ok(())
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
    let mut options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("image")
            .size_limit(64_000_000)
            .content_type_by_string(Some(mime::IMAGE_STAR))
            .map_err(to_debug)?,
        MultipartFormDataField::text("refers_to_id"),
    ]);
    options.max_data_bytes=64_000_000;
    let multipart_form_data = MultipartFormData::parse(content_type, data, options)
        
        .await
        .map_err(to_debug)?;

    // cast data to normal values
    let photo_path = multipart_form_data
        .files
        .get("image")
        .ok_or(str_to_debug("image field not found"))?
        .get(0)
        .ok_or(str_to_debug("err"))?; //at drop it cleans the file

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

    //generate unique name and convert
    let new_pos = unique_path(Path::new(&config.media_folder), Path::new("jpg"));
    convert_image(&photo_path.path, &new_pos)?;

    let name = new_pos
        .strip_prefix(&config.media_folder)
        .map_err(to_debug)?
        .to_str()
        .ok_or(str_to_debug("err"))?
        .to_string();

    // try to save it in database
    save_image(connection, name.clone(), id)
        .map_err(|x| {
            let _ = fs::remove_file(new_pos);
            x
        })
        .await?;

    Ok(())
}

#[get("/image/list/<marker_id>")]
pub async fn list_image(
    marker_id: i64,
    connection: Db,
) -> Result<Json<Vec<i64>>, Debug<Box<dyn Error>>> {
    let res: Vec<MarkerImage> = connection
        .run(move |conn| {
            marker_images::table
                .filter(marker_images::refers_to.eq(marker_id))
                .load::<MarkerImage>(conn)
        })
        .await
        .map_err(to_debug)?;
    Ok(Json(res.iter().map(|x| x.id.unwrap()).collect()))
}

#[get("/image/<image_id>")]
pub(crate) async fn get_image(
    image_id: i64,
    connection: Db,
    config: &State<InsignoConfig>,
) -> Option<NamedFile> {
    let res: MarkerImage = connection
        .run(move |conn| {
            marker_images::table
                .find(image_id)
                .load::<MarkerImage>(conn)
        })
        .await
        .map_err(to_debug)
        .ok()?
        .get(0)?
        .clone();
    let mut path = PathBuf::new();
    path.push(config.media_folder.clone());
    path.push(res.path);

    //print!("{:?}", path);
    NamedFile::open(path).await.ok()
}
/* 
#[cfg(test)]
mod test {
    use rocket::local::asynchronous::Client;
    use crate::rocket;
    
    #[rocket::async_test]
    async fn test_marker_add_image() {
        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");
        // try to get types list
        let response = client.post("/map/image/add").dispatch().await;

    }
}*/