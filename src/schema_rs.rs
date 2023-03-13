use std::io::Write;

use chrono::Utc;
use diesel::backend::Backend;
use diesel::deserialize;
use diesel::pg::Pg;
use diesel::serialize;
use diesel::serialize::Output;
use diesel::types::FromSql;
use diesel::types::ToSql;
use postgis::ewkb::Point;
use postgis_diesel::PointC;
use postgis_diesel::sql_types::Geometry;
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

#[derive(AsExpression, FromSqlRow, Debug, Clone)]
#[sql_type = "Geometry"]
pub struct InsignoPoint {
    point: PointC<Point>,
}


//pub struct PostId{pub value: Poin}

impl<DB: Backend<RawValue=[u8]>> FromSql<Geometry, DB> for InsignoPoint {
  fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
    <PointC<Point> as FromSql<Geometry, Pg>>::from_sql(bytes).map(|value| InsignoPoint{ point: value })
  }
}

impl ToSql<Geometry, Pg> for InsignoPoint {
  fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
    <PointC<Point> as ToSql<Geometry, Pg>>::to_sql(&self.point, out)
  }
}

impl Serialize for InsignoPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Point", 3)?;
        s.serialize_field("x", &self.point.v.x)?;
        s.serialize_field("y", &self.point.v.y)?;
        s.serialize_field("srid", &self.point.v.srid)?;
        s.end()
    }
}

impl InsignoPoint{
    pub fn new(x: f64, y: f64)-> Self{
        InsignoPoint{point: PointC {
            v: Point {
                x: x,
                y: y,
                srid: Some(4326),
            },
        },}

    }
}

#[derive(Clone, Queryable, Debug, Serialize, Insertable)]
#[diesel(table_name = marker)]
pub struct Marker {
    #[diesel(deserialize_as = "i64")]
    pub id: Option<i64>,

    pub point: InsignoPoint,

    #[diesel(deserialize_as = "chrono::DateTime<Utc>")]
    pub creation_date: Option<chrono::DateTime<Utc>>,

    pub resolution_date: Option<chrono::DateTime<Utc>>,

    pub created_by: i64,
    pub solved_by: Option<i64>,
    pub marker_types_id: i64,
}
/*
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
}*/

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
    pub name: String,
    pub password: String,
    pub is_admin: bool,
    pub points: f64,
}

#[derive(Clone, Queryable, Insertable, Debug)]
pub struct UserSession{
    pub user_id: i64,
    pub token: String,
    pub refresh_date: chrono::DateTime<Utc>,
}