use std::path::{Path, PathBuf};

use rocket::{fs::NamedFile, serde::json::Json, State};

use crate::{utils::InsignoError, InsignoConfig};

#[get("/intro")]
pub fn get_intro_image_links(
    config: &State<InsignoConfig>,
) -> Result<Json<Vec<String>>, InsignoError> {
    return Ok(Json(
        config
            .intro_images
            .iter()
            .map(|filename| format!("/resource/intro/{filename}"))
            .collect(),
    ));
}

// note: this is temporary and will be replaced with a more general way to serve static files
#[get("/intro/<path..>")]
pub async fn get_intro_image(
    path: PathBuf,
    config: &State<InsignoConfig>,
) -> Result<NamedFile, InsignoError> {
    let path_str = path
        .to_str()
        .ok_or_else(|| InsignoError::new(400).both("invalid path string"))?;

    // PathBuf prevents path traversal attacks,
    // but still check whether the requested image is an intro image
    if !config.intro_images.contains(&path_str.to_string()) {
        return Err(InsignoError::new(400).both("not an intro image"));
    }

    NamedFile::open(Path::new("static/").join(path))
        .await
        .map_err(|e| InsignoError::new(404).debug(e))
}
