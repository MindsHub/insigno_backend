use rocket::{form::Form, http::ContentType, State};

use crate::{
    auth::{
        user::{Authenticated, User},
        validation::ScryptSemaphore,
    },
    db::Db,
    mail::MailBuilder,
    pending::{generate_token, Pending, PendingAction},
    utils::InsignoError,
    InsignoConfig,
};

use super::{login::LoginInfo, scrypt::scrypt_simple, user::UserDiesel};

#[get("/delete_account_web")]
pub fn delete_account_web_form() -> (ContentType, String) {
    // no need for CSRF tokens here, since no cookies are used in the POST below,
    // and the user must authenticate anew even if they were already logged in
    // TODO use template
    (
        ContentType::HTML,
        r#"<h1>Eliminazione account Insigno</h1>
        Dopo aver premuto su "Invia" ti arriverà una mail in cui dovrai confermare l'eliminazione dell'account.
        <br/>
        <b>Ricordati di controllare anche nello spam!</b>
        <br/>
        <br/>
        <form method="post" action="/delete_account_web">
            <label for="email">Email:</label>
            <input id="email" name="email" type="email" />
            <br/>
            <label for="password">Password:</label>
            <input id="password" name="password" type="password" />
            <br/>
            <button type="submit">Invia</button>
        </form>
        "#.to_string()
    )
}

#[post("/delete_account_web", format = "form", data = "<login_info>")]
pub async fn delete_account_web(
    login_info: Form<LoginInfo>,
    scrypt_sem: &State<ScryptSemaphore>,
    mailer: &State<MailBuilder>,
    connection: Db,
) -> Result<(ContentType, String), InsignoError> {
    let user = login_info
        .into_inner()
        .into_authenticated_user(scrypt_sem, &connection)
        .await?;
    add_pending_delete(mailer, user, connection).await?;
    Ok((ContentType::HTML, "<h1>Controlla la mail, e guarda anche nello spam, per completare l'eliminazione dell'account!</h1>".to_string()))
}

#[post("/delete_account")]
pub async fn delete_account(
    mailer: &State<MailBuilder>,
    user: Result<User<Authenticated>, InsignoError>,
    connection: Db,
) -> Result<(), InsignoError> {
    add_pending_delete(mailer, user?, connection).await
}

async fn add_pending_delete(
    mailer: &State<MailBuilder>,
    user: User<Authenticated>,
    connection: Db,
) -> Result<(), InsignoError> {
    let mut pend = Pending::new(PendingAction::DeleteUser(user.id.unwrap()));
    pend.insert(&connection).await?;
    let link = format!("https://insigno.mindshub.it/verify/{}", pend.token);
    mailer
        .send_delete_account_mail(&user.email, &user.name, &link)
        .await
        .map_err(|e| InsignoError::new(500).debug(e))?;
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
        Ok((ContentType::HTML, "Account eliminato".to_string()))
    } else {
        Err(InsignoError::new(500).debug("wrong call"))
    }
}
