use chrono::Utc;
use postgis::ewkb::Point;
use postgis_diesel::PointC;
use rocket::serde::{Deserialize, Serialize};
use serde::ser::SerializeStruct;

use crate::schema_sql::*;
use crate::utils::*;

#[derive(Serialize, Clone, Queryable, Debug)]
#[diesel(table_name = "marker_types")]
pub struct MarkerType {
    id: i64,
    name: String,
    points: f64,
}

#[derive(Clone, Queryable, Insertable, Debug)]
#[diesel(table_name = marker)]
pub struct Marker {
    #[diesel(deserialize_as = "i64")]
    pub id: Option<i64>,
    pub point: PointC<Point>,
    #[diesel(deserialize_as = "chrono::DateTime<Utc>")]
    pub creation_date: Option<chrono::DateTime<Utc>>,

    //#[diesel(deserialize_as = "chrono::DateTime<Utc>")]
    pub resolution_date: Option<chrono::DateTime<Utc>>,

    pub created_by: i64,
    pub solved_by: Option<i64>,
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
        s.serialize_field("resolution_date", &self.resolution_date)?;
        s.serialize_field("created_by", &self.created_by)?;
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
#[derive(Debug, Clone, Default, QueryId, Serialize, Deserialize, Insertable, Queryable, QueryableByName)]
#[table_name = "users"]
//#[derive(Queryable, Clone,  SqlType)]
pub struct User {
    
    pub id: Option<i64>,
    pub email: String,
    pub password: String,
    pub is_admin: bool,
    pub points: f64,
}
/*id-> Nullable<BigInt>,
        user_id -> BigInt,
        token -> Text,
        date -> Timestamptz, */

#[derive(Clone, Queryable, Insertable, Debug)]
pub struct UserSession{
    pub user_id: i64,
    pub token: String,
    pub refresh_date: chrono::DateTime<Utc>,
}