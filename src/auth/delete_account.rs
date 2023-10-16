use rocket::http::ContentType;

use crate::{
    auth::user::{Authenticated, User},
    db::Db,
    pending::{generate_token, Pending, PendingAction},
    utils::InsignoError,
    InsignoConfig,
};

use super::{scrypt::scrypt_simple, user::UserDiesel};

#[post("/delete_account")]
pub async fn delete_account(
    user: Result<User<Authenticated>, InsignoError>,
    connection: Db,
) -> Result<(), InsignoError> {
    let user = user?;
    let mut pend = Pending::new(PendingAction::DeleteUser(user.id.unwrap()));
    pend.insert(&connection).await?;
    Ok(())
}

pub async fn complete_delete(
    pend: PendingAction,
    connection: &Db,
    config: &InsignoConfig,
) -> Result<(ContentType, String), InsignoError> {
    if let PendingAction::DeleteUser(user_id) = pend {
        let permit = config.scrypt.clone().await;
        let params = permit.get_params();
        let mut user = UserDiesel::get_by_id(connection, user_id).await?;
        user.email = scrypt_simple(&user.email, &params)
            .or(Err(InsignoError::new(500).debug("error in scrypt")))?;
        user.password = "".to_string();
        user.name = format!("ANON_{}", generate_token());
        user.update(connection).await?;
        Ok((ContentType::HTML, "registrazione completata".to_string()))
    } else {
        Err(InsignoError::new(500).debug("wrong call"))
    }
}
