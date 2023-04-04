use serde::Serialize;

use super::validation::{Email, Password};

#[derive(Serialize, FromForm)]
pub struct LoginInfo {
    pub email: String,
    pub password: String,
}
impl Email for LoginInfo {
    fn get_email(&mut self) -> &mut String {
        &mut self.email
    }
}
impl Password for LoginInfo {
    fn get_password(&mut self) -> &mut String {
        &mut self.password
    }
}
