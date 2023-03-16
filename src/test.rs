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
    let data = "name=test@gmail.com&password=Testtes1";
    let response = client
        .post("/signup")
        .header(ContentType::Form)
        .body(data)
        .dispatch();
    assert_eq!(response.await.status(), Status::Ok);
}
