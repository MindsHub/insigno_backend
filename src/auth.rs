use diesel::{PgConnection, Connection, insert_into, update, QueryDsl, RunQueryDsl};
use rocket::{fairing::AdHoc, form::Form, get, post, routes, Route};
use rocket_auth::{Auth, Error, Login, Signup, Users, DBConnection, Result, User};
use diesel::*;

use crate::schema::users;

pub struct UserConnection(pub diesel::PgConnection);
unsafe impl Sync for UserConnection {}
#[derive(Queryable, Clone)]
struct MyUser{
    id: i32,
    email: String,
    password: String,
    is_admin: bool,
}
impl Into<User> for MyUser{
    fn into(self) -> User {
        User{id: self.id, email: self.email, password: self.password, is_admin: self.is_admin}
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
        
        update(dslUsers.find(user.id.clone())).set(( users::email.eq( user.email().clone()), users::password.eq(user.password.clone()), users::is_admin.eq(user.is_admin))).execute(&self.0)?;
        
        Ok(())
    }

    async fn delete_user_by_id(&self, user_id: i32) -> Result<()> {

        use users::dsl::users as dslUsers;
        delete(dslUsers.find(user_id)).execute(&self.0)?;
        
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
        let z = dslUsers.find(user_id).load::<MyUser>(&self.0)?[0].clone();

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
        let y = PgConnection::establish(&"postgres://mindshub:test@localhost:5432/insigniorocketdb").unwrap();
        let y = UserConnection{0: y};
        let users: Users = y.into();
        rocket.manage(users)
    })
}
