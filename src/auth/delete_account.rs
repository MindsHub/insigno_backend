use crate::{auth::user::{Authenticated, User}, utils::InsignoError};

#[post("/delete_account")]
pub fn delete_account(user: Result<User<Authenticated>, InsignoError>) -> Result<(), InsignoError> {
    let _user = user?;
    Ok(())
}