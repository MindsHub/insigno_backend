//use rocket::tokio::fs;

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;

use super::marker_image::MarkerImage;
use super::marker_report::ImageToReport;
use crate::auth::user::YesReview;
use crate::auth::user::Authenticated;

use crate::auth::user::User;
use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;
use crate::map::marker_image::marker_images;
use diesel::insert_into;
use diesel::QueryDsl;

use rocket::data::Limits;
use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::futures::TryFutureExt;

use rocket::serde::json::Json;
use rocket::Data;
use rocket::{http::ContentType, State};
use rocket_multipart_form_data::*;
use serde::Deserialize;
use serde::Serialize;

use crate::db::*;
use crate::schema_rs::*;
use crate::schema_sql::*;
use crate::utils::*;
use crate::InsignoConfig;
use std::str;

use super::marker_report::MarkerReport;
fn convert_image(input: &Path, output: &Path) -> Result<(), InsignoError> {
    if input.exists() {
        let raw_out = process::Command::new("ffmpeg")
            .args([
                "-i",
                input
                    .to_str()
                    .ok_or(InsignoError::new(500).debug("invalid path"))?,
                "-vf",
                "scale=w=2500:h=2500:force_original_aspect_ratio=decrease",
                output
                    .to_str()
                    .ok_or(InsignoError::new(500).debug("invalid path"))?,
            ])
            .output()
            .map_err(|e| InsignoError::new(500).debug(e))?;
        if !raw_out.status.success() {
            return Err(
                InsignoError::new(500).debug(str::from_utf8(&raw_out.stderr).unwrap()), //str::from_utf8(&raw_out.stderr).unwrap(),
            )?;
        }
        Ok(())
    } else {
        Err(InsignoError::new(500).debug("input path does not exits"))?
    }
}

async fn save_image(connection: Db, name: String, id: i64) -> Result<(), InsignoError> {
    let img = MarkerImage {
        id: None,
        path: name,
        refers_to: id,
        approved: false,
    };

    connection
        .run(move |conn| {
            use marker_images::dsl::marker_images as mi;
            insert_into(mi).values(&img).get_result::<MarkerImage>(conn)
        })
        .await
        .map_err(|e| InsignoError::new(500).debug(e))?;

    Ok(())
}

#[post("/image/add", data = "<data>")]
pub(crate) async fn add_image(
    content_type: &ContentType,
    data: Data<'_>,
    user: Result<User<Authenticated>, InsignoError>,
    connection: Db,
    config: &State<InsignoConfig>,
    limits: &Limits,
) -> Result<(), InsignoError> {
    let user = user?;
    // parse multipart data
    let mut options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("image")
            .size_limit(limits.get("data-form").unwrap().as_u64())
            .content_type_by_string(Some(mime::IMAGE_STAR))
            .map_err(|e| InsignoError::new(500).debug(e))?,
        MultipartFormDataField::text("refers_to_id"),
    ]);
    options.max_data_bytes = limits.get("data-form").unwrap().as_u64();
    let multipart_form_data = MultipartFormData::parse(content_type, data, options)
        .await
        .map_err(|e| InsignoError::new(500).debug(e))?;

    // cast data to normal values
    let photo_path = multipart_form_data
        .files
        .get("image")
        .ok_or(InsignoError::new(500).debug("image field not found"))? //str_to_debug("image field not found"))?
        .get(0)
        .ok_or(InsignoError::new(500).debug("err"))?; //str_to_debug("err"))?; //at drop it cleans the file

    let id = multipart_form_data
        .texts
        .get("refers_to_id")
        .ok_or(InsignoError::new(500).debug("image field not found"))?[0]
        .text
        .parse::<i64>()
        .map_err(|e| InsignoError::new(500).debug(e))?;

    let user_id = user.get_id();

    // check if user own the marker
    connection
        .run(move |conn| {
            markers::table
                .filter(markers::id.eq(id))
                .filter(markers::created_by.eq(user_id))
                .load::<Marker>(conn)
        })
        .await
        .map_err(|e| InsignoError::new(500).debug(e))?;

    //generate unique name and convert
    let new_pos = unique_path(Path::new(&config.media_folder), Path::new("jpg"));
    convert_image(&photo_path.path, &new_pos)?;

    let name = new_pos
        .strip_prefix(&config.media_folder)
        .map_err(|e| InsignoError::new(500).debug(e))?
        .to_str()
        .ok_or(InsignoError::new(500).debug("err"))?
        .to_string();

    // try to save it in database
    save_image(connection, name.clone(), id)
        .map_err(|x| {
            let _ = fs::remove_file(new_pos); //sync version
            x
        })
        .await?;

    Ok(())
}

pub async fn _list_image(marker_id: i64, connection: &Db) -> Result<Vec<i64>, InsignoError> {
    let res: Vec<MarkerImage> = connection
        .run(move |conn| {
            marker_images::table
                .filter(marker_images::refers_to.eq(marker_id))
                .load::<MarkerImage>(conn)
        })
        .await
        .map_err(|x| InsignoError::new(404).debug(x))?;
    Ok(res.iter().map(|x| x.id.unwrap()).collect())
}

#[get("/image/list/<marker_id>")]
pub async fn list_image(marker_id: i64, connection: Db) -> Result<Json<Vec<i64>>, InsignoError> {
    Ok(Json(_list_image(marker_id, &connection).await?))
}

#[get("/image/<image_id>")]
pub(crate) async fn get_image(
    image_id: i64,
    connection: Db,
    config: &State<InsignoConfig>,
) -> Result<NamedFile, InsignoError> {
    let res: MarkerImage = connection
        .run(move |conn| {
            marker_images::table
                .find(image_id)
                .load::<MarkerImage>(conn)
        })
        .await
        .map_err(|e| InsignoError::new(500).debug(e))?
        .get(0)
        .ok_or(InsignoError::new(404).debug("image field not found"))?
        .clone();
    let mut path = PathBuf::new();
    path.push(config.media_folder.clone());
    path.push(res.path);

    //print!("{:?}", path);
    NamedFile::open(path)
        .await
        .map_err(|e| InsignoError::new(500).debug(e))
}

#[allow(unused_variables)]
#[get("/image/to_review")]
pub(crate) async fn get_to_review(
    connection: Db,
    user: Result<User<Authenticated, YesReview>, InsignoError>,
) -> Result<Json<Vec<ImageToReport>>, InsignoError> {
    let user = user?;

    let images = ImageToReport::get_to_report(&connection).await?;
    Ok(Json(images))
}

#[derive(Deserialize, Serialize, FromForm)]
pub(crate) struct ReviewVerdict {
    verdict: String,
}

#[post("/image/review/<image_id>", data = "<verdict>")]
pub(crate) async fn review(
    image_id: i64,
    connection: Db,
    config: &State<InsignoConfig>,
    user: Result<User<Authenticated, YesReview>, InsignoError>,
    verdict: Form<ReviewVerdict>,
) -> Result<(), InsignoError> {
    let user = user?;

    let verdict = verdict.verdict.trim().to_ascii_lowercase();
    match verdict.as_str() {
        "ok" => {
            MarkerImage::approve(&connection, image_id).await?;
        }
        "delete" => {
            MarkerImage::delete(&connection, image_id, config).await?;
        }
        "delete_report" => {
            let image = MarkerImage::delete(&connection, image_id, config).await?;
            MarkerReport::report(&connection, user.get_id(), image.id.unwrap()).await?;
        }
        "skip" => {}
        _ => {
            return Err(InsignoError::new(422).both("opzione non riconosciuta"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{
        rocket,
        test::{test_add_image, test_add_point, test_reset_db, test_signup},
    };
    use rocket::{http::Status, local::asynchronous::Client};

    #[rocket::async_test]
    async fn test_marker_add_image() {
        test_reset_db();

        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");

        let response = client.get("/map/image/list/1").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().await.unwrap(), "[]".to_string());

        let response = client.get("/map/image/1").dispatch().await;
        assert_eq!(response.status(), Status::NotFound);

        let _id = test_signup(&client).await;
        test_add_point(&client).await;
        test_add_image(1, "./tests/test_data/add_image.jpg", &client).await;

        let response = client.get("/map/image/list/1").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().await.unwrap(), "[1]".to_string());

        let response = client.get("/map/image/1").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }
}
