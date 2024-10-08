use chrono::Utc;

use diesel::sql_types::*;
use postgis_diesel::sql_types::Geometry;
use postgis_diesel::types::Point;
use rocket::serde::Serialize;
use serde::ser::SerializeStruct;

use crate::schema_sql::*;

define_sql_function!(fn st_transform(g: Geometry, srid: BigInt)-> Geometry);
define_sql_function!(fn st_dwithin(g1: Geometry, g2: Geometry, dist: Double) ->  Bool); // "Represents the postgis_sql distance() function"
define_sql_function!(fn resolve_marker(marker_id: BigInt, user_id: BigInt));

#[derive(Serialize, Clone, Queryable, Debug)]
#[diesel(table_name = marker_types)]
pub struct MarkerType {
    id: i64,
    name: String,
    points: f64,
}

#[derive(Clone, Debug)]
pub struct InsignoPoint {
    point: Point,
}

impl Serialize for InsignoPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Point", 3)?;
        s.serialize_field("x", &self.point.x)?;
        s.serialize_field("y", &self.point.y)?;
        s.serialize_field("srid", &self.point.srid)?;
        s.end()
    }
}

impl From<Point> for InsignoPoint {
    fn from(value: Point) -> Self {
        Self { point: value }
    }
}

impl From<InsignoPoint> for Point {
    fn from(val: InsignoPoint) -> Self {
        val.point
    }
}
/*
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
}*/

#[derive(Clone, Queryable, Debug, Serialize, Insertable, QueryableByName)] //
#[diesel(table_name = markers)]
pub struct Marker {
    #[diesel(sql_type = Nullable<BigInt>)]
    pub id: Option<i64>,

    #[diesel(sql_type = Geometry)]
    #[diesel(serialize_as = Point)]
    #[diesel(deserialize_as = Point)]
    pub point: InsignoPoint,

    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub creation_date: Option<chrono::DateTime<Utc>>,

    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub resolution_date: Option<chrono::DateTime<Utc>>,

    #[diesel(sql_type = BigInt)]
    pub created_by: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub solved_by: Option<i64>,
    #[diesel(sql_type = BigInt)]
    pub marker_types_id: i64,
}

#[derive(Clone, Queryable, Insertable, Debug)]
pub struct UserSession {
    pub user_id: i64,
    pub token: String,
    pub refresh_date: chrono::DateTime<Utc>,
}
