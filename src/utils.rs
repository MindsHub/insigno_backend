use postgis_diesel::types::*;
use serde::{Serialize, ser::SerializeStruct};
use  postgis_diesel::sql_types::*;
use diesel::sql_types::*;
pub struct InsignoPoint{
    point: Point,
}

impl From<Point> for InsignoPoint{
    fn from(point: Point) -> Self {
        InsignoPoint {point}
    }
}

impl Serialize for InsignoPoint{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            let mut s = serializer.serialize_struct("Point", 3)?;
            s.serialize_field("x", &self.point.x)?;
            s.serialize_field("y", &self.point.y)?;
            s.serialize_field("srid", &self.point.srid)?;
            s.end()
    }
}

pub struct InsignoTimeStamp{
    dtm: chrono::NaiveDateTime,
}

impl From<chrono::NaiveDateTime> for InsignoTimeStamp{
    fn from(dtm: chrono::NaiveDateTime) -> Self {
        InsignoTimeStamp { dtm }
    }
}

impl Serialize for InsignoTimeStamp{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            let mut s = serializer.serialize_struct("DateTime", 3)?;
            s.serialize_field("dtm", &self.dtm.to_string())?;
            s.end()
    }
}

sql_function!(fn st_transform(g: Geometry, srid: Integer)-> Geometry);
sql_function!(fn st_dwithin(g1: Geometry, g2: Geometry, dist: Double) ->  Bool); // "Represents the postgis_sql distance() function"