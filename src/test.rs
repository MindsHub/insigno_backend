use std::process::Command;

use rocket::{
    http::{ContentType, Status},
    local::asynchronous::Client,
};
#[allow(dead_code)]
pub fn test_reset_db() {
    let success = Command::new("diesel")
        .args(["database", "reset"])
        .output()
        .unwrap()
        .status
        .success();
    assert!(success);
}
#[allow(dead_code)]
pub async fn test_signup(client: &Client) {
    let data = "email=test@gmail.com&password=Testtes1";
    let response = client
        .post("/signup")
        .header(ContentType::Form)
        .body(data)
        .dispatch();
    assert_eq!(response.await.status(), Status::Ok);
}
