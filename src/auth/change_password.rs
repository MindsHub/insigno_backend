use rocket::{form::Form, State};

use crate::{db::Db, pending::{Pending, generate_token, PendingAction}, utils::InsignoError, mail::MailBuilder};


#[derive(FromForm)]
pub struct ChangePasswordRequest{
    user_id: i64,
}

#[post("/change_password", format = "form", data = "<change_password_request>")]
pub async fn change_password(
    db: Db,
    change_password_request: Form<ChangePasswordRequest>,
    mailer: &State<MailBuilder>
) -> Result<String, InsignoError> {
    let mut pending: Pending = Pending::new(PendingAction::ChangePassword(change_password_request.user_id));
    pending.insert(&db).await?;

    Ok("Abbiamo inviato una mail all'utente interessato".to_string())
}