use chrono::Utc;
use postgis::ewkb::Point;
use postgis_diesel::PointC;
use rocket::serde::Serialize;
use serde::ser::SerializeStruct;

use crate::schema_sql::*;
use crate::utils::*;
use rocket_auth::User as AUser;

#[derive(Serialize, Clone, Queryable, Debug)]
#[diesel(table_name = "marker_types")]
pub struct MarkerType {
    id: i64,
    name: String,
}

#[derive(Clone, Queryable, Insertable, Debug)]
#[diesel(table_name = marker)]
pub struct Marker {
    #[diesel(deserialize_as = "i64")]
    pub id: Option<i64>,
    pub point: PointC<Point>,
    #[diesel(deserialize_as = "chrono::DateTime<Utc>")]
    pub creation_date: Option<chrono::DateTime<Utc>>,
    pub created_by: i64,
    pub marker_types_id: i64,
}

impl Serialize for Marker {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Marker", 4)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("point", &InsignoPoint::from(self.point))?;
        s.serialize_field("creation_date", &self.creation_date)?;
        s.serialize_field("marker_types_id", &self.marker_types_id)?;
        s.end()
    }
}

#[derive(Clone, Queryable, Insertable, Debug)]
#[diesel(table_name = image)]
pub struct MarkerImage {
    #[diesel(deserialize_as = "i64")]
    pub id: Option<i64>,
    pub path: String,
    pub refers_to: i64,
}

#[derive(Queryable, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    password: String,
    pub is_admin: bool,
    pub points: i64,
}

impl From<User> for AUser {
    fn from(val: User) -> Self {
        AUser {
            id: val.id as i32,
            email: val.email,
            password: val.password,
            is_admin: val.is_admin,
        }
    }
}
