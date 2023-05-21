use rocket::{form::Form, State};

use crate::{db::Db, pending::{Pending, PendingAction}, utils::InsignoError, mail::MailBuilder};

use super::user::User;


#[derive(FromForm)]
pub struct ChangePasswordRequest{
    email: String,
}

#[post("/change_password", format = "form", data = "<change_password_request>")]
pub async fn change_password(
    db: Db,
    change_password_request: Form<ChangePasswordRequest>,
    mailer: &State<MailBuilder>
) -> Result<String, InsignoError> {
    
    let _: Result<(), InsignoError> =  async move{ 
        let user = User::get_by_email(&db, change_password_request.email.clone()).await?;
        let mut pending: Pending = Pending::new(PendingAction::ChangePassword(user.id.unwrap()));
        pending.insert(&db).await?;
        let link=format!("https://insigno.mindshub.it/verify/{}", pending.token);
        mailer.send_change_password_mail(&user.email, &user.name, &link).await.map_err(|_| InsignoError::new_code(500))?;
        Ok(())
    }.await;
    
    Ok("Abbiamo inviato una mail all'utente interessato".to_string())
}