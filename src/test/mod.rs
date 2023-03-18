use std::process::Command;

use rocket::{
    http::{ContentType, Status},
    local::asynchronous::Client,
};

pub fn test_reset_db() {
    let success = Command::new("diesel")
        .args(["database", "reset"])
        .output()
        .unwrap()
        .status
        .success();
    assert!(success);
}

pub async fn test_signup(client: &Client) {
    let data = "name=IlMagicoTester&password=Testtes1!";
    let response = client
        .post("/signup")
        .header(ContentType::Form)
        .body(data)
        .dispatch();
    assert_eq!(response.await.status(), Status::Ok);
}

pub async fn test_add_point(client: &Client) {
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
}

pub async fn test_add_image(marker_id: i64, path: &str, c: &Client) {
    use form_data_builder::FormData;
    let mut form = FormData::new(Vec::new()); // use a Vec<u8> as a writer;

    form.write_path("image", path, "image/jpg").unwrap();
    form.write_field("refers_to_id", &marker_id.to_string())
        .unwrap();
    let y = form.finish().unwrap(); // returns the writer
    let temp_str = form.content_type_header();
    let w: Vec<&str> = temp_str.split("/").collect();

    let response = c
        .post("/map/image/add")
        .header(ContentType::new(w[0].to_string(), w[1].to_string()))
        .body(y)
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);
}
