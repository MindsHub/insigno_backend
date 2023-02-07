use diesel::{PgConnection, Connection, insert_into, update, QueryDsl, RunQueryDsl};
use rocket::{fairing::AdHoc, form::Form, get, post, routes, Route};
use rocket_auth::{Auth, Error, Login, Signup, Users, DBConnection, Result, User};
use diesel::*;
use rocket_sync_db_pools::Config;

use crate::schema::users;

pub struct UserConnection(pub diesel::PgConnection);
unsafe impl Sync for UserConnection {}
#[derive(Queryable, Clone)]
struct MyUser{
    id: i64,
    email: String,
    password: String,
    is_admin: bool,
}
impl From<MyUser> for User{
    fn from(val: MyUser) -> Self {
        User{id: val.id as i32, email: val.email, password: val.password, is_admin: val.is_admin}
    }
}

#[rocket::async_trait]
impl DBConnection for UserConnection{
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    async fn create_user(&self, email: &str, hash: &str, is_admin: bool) -> Result<()> {
        let email = email.to_string();
        let hash = hash.to_string();
        
        use users::dsl::users as dslUsers;
        use crate::diesel::ExpressionMethods;
        insert_into(dslUsers).values((users::email.eq(email), users::password.eq(hash), users::is_admin.eq(is_admin))).execute(&self.0)?;
        Ok(())
    }

    async fn update_user(&self, user: &User) -> Result<()> {
        let user = user.clone();
        
        use users::dsl::users as dslUsers;
        
        update(dslUsers.find(user.id as i64)).set(( users::email.eq( user.email().to_string()), users::password.eq(user.password), users::is_admin.eq(user.is_admin))).execute(&self.0)?;
        
        Ok(())
    }

    async fn delete_user_by_id(&self, user_id: i32) -> Result<()> {

        use users::dsl::users as dslUsers;
        delete(dslUsers.find(user_id as i64)).execute(&self.0)?;
        
        Ok(())
    }
    async fn delete_user_by_email(&self, email: &str) -> Result<()> {
        let email = email.to_string();
        use users::dsl::users as dslUsers;
        delete(dslUsers).filter(users::email.eq(email)).execute(&self.0)?;
        Ok(())
    }
    async fn get_user_by_id(&self, user_id: i32) -> Result<User> {
        use users::dsl::users as dslUsers;
        let z = dslUsers.find(user_id as i64).load::<MyUser>(&self.0)?[0].clone();

        Ok(z.into())
    }
    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let email = email.to_string();
        use users::dsl::users as dslUsers;
        let z = dslUsers.filter(users::email.eq(email)).first::<MyUser>(&self.0)?;

        Ok(z.into())
    }
}


#[post("/signup", data = "<form>")]
async fn signup(form: Form<Signup>, auth: Auth<'_>) -> Result<&'static str, Error> {
    auth.signup(&form).await?;
    auth.login(&form.into()).await?;
    Ok("You signed up.")
}

#[post("/login", data = "<form>")]
async fn login(form: Form<Login>, auth: Auth<'_>) -> Result<&'static str, Error> {
    auth.login(&form).await?;
    Ok("You're logged in.")
}

#[get("/logout")]
fn logout(auth: Auth<'_>) -> Result<(), Error> {
    auth.logout()?;
    Ok(())
}

pub fn get_routes() -> Vec<Route> {
    routes![signup, login, logout]
}

pub async fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel Authentication Stage", |rocket| async {
        let y = Config::from("db", &rocket).unwrap();
        let y = PgConnection::establish(&y.url).unwrap();
        let y = UserConnection{0: y};
        let users: Users = y.into();
        rocket.manage(users)
    })
}
