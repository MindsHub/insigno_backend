use rocket::{
    http::Status,
    request::{self, FromRequest},
};
use serde::Serialize;

use crate::utils::InsignoError;

use super::{authenticated_user::AuthenticatedUser, user::User};

pub struct AdminUser {
    user: AuthenticatedUser,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = InsignoError;

    async fn from_request(request: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
        let user = AuthenticatedUser::from_request(request).await.unwrap();
        if !user.as_ref().is_admin {
            return request::Outcome::Failure((Status::Forbidden, InsignoError::new_code(401)));
        }
        request::Outcome::Success(AdminUser { user })
    }
}
impl AsRef<AuthenticatedUser> for AdminUser {
    fn as_ref(&self) -> &AuthenticatedUser {
        &self.user
    }
}

impl AsRef<User> for AdminUser {
    fn as_ref(&self) -> &User {
        self.user.as_ref()
    }
}

impl Serialize for AdminUser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.user.serialize(serializer)
    }
}
