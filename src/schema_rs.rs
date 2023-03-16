use std::io::Write;

use chrono::Utc;
use diesel::backend::Backend;
use diesel::deserialize;
use diesel::pg::Pg;
use diesel::serialize;
use diesel::serialize::Output;
use diesel::sql_types::*;
use diesel::types::{FromSql, ToSql};
use postgis::ewkb::Point;
use postgis_diesel::sql_types::Geometry;
use postgis_diesel::PointC;
use rocket::serde::{Deserialize, Serialize};
use serde::ser::SerializeStruct;

use crate::schema_sql::*;

sql_function!(fn st_transform(g: Geometry, srid: BigInt)-> Geometry);
sql_function!(fn st_dwithin(g1: Geometry, g2: Geometry, dist: Double) ->  Bool); // "Represents the postgis_sql distance() function"
sql_function!(fn resolve_marker(marker_id: BigInt, user_id: BigInt));

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

impl<DB: Backend<RawValue = [u8]>> FromSql<Geometry, DB> for InsignoPoint {
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        <PointC<Point> as FromSql<Geometry, Pg>>::from_sql(bytes)
            .map(|value| InsignoPoint { point: value })
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

impl InsignoPoint {
    pub fn new(x: f64, y: f64) -> Self {
        InsignoPoint {
            point: PointC {
                v: Point {
                    x,
                    y,
                    srid: Some(4326),
                },
            },
        }
    }
}

#[derive(Clone, Queryable, Debug, Serialize, Insertable, QueryableByName)]
#[diesel(table_name = "markers")]
pub struct Marker {
    #[sql_type = "Nullable<BigInt>"]
    pub id: Option<i64>,
    #[sql_type = "Geometry"]
    pub point: InsignoPoint,

    #[sql_type = "Nullable<Timestamptz>"]
    pub creation_date: Option<chrono::DateTime<Utc>>,

    #[sql_type = "Nullable<Timestamptz>"]
    pub resolution_date: Option<chrono::DateTime<Utc>>,

    #[sql_type = "BigInt"]
    pub created_by: i64,
    #[sql_type = "Nullable<BigInt>"]
    pub solved_by: Option<i64>,
    #[sql_type = "BigInt"]
    pub marker_types_id: i64,
}

#[derive(Clone, Queryable, Insertable, Debug)]
#[diesel(table_name = image)]
pub struct MarkerImage {
    #[diesel(deserialize_as = "i64")]
    pub id: Option<i64>,
    pub path: String,
    pub refers_to: i64,
}
#[derive(
    Debug, Clone, Default, QueryId, Serialize, Deserialize, Insertable, Queryable, QueryableByName,
)]
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
pub struct UserSession {
    pub user_id: i64,
    pub token: String,
    pub refresh_date: chrono::DateTime<Utc>,
}
/*marker_reports(id){
    id -> BigInt,
    from -> BigInt,
    reported_marker -> BigInt,
} */
#[derive(Clone, Queryable, Insertable, Debug, QueryableByName)]
#[table_name = "marker_reports"]
pub struct MarkerReport {
    pub id: Option<i64>,
    pub user_f: i64,
    pub reported_marker: i64,
}
