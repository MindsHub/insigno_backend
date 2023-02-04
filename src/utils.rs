use std::path::{Path, PathBuf};

use postgis::ewkb::Point;
use postgis_diesel::*;
use rand::{Rng, distributions::Alphanumeric};
use serde::{Serialize, ser::SerializeStruct};
use  postgis_diesel::sql_types::*;
use diesel::sql_types::*;

pub struct InsignoPoint{
    point: PointC<Point>,
}

impl From<PointC<Point>> for InsignoPoint{
    fn from(point: PointC<Point>) -> Self {
        InsignoPoint {point}
    }
}

impl Serialize for InsignoPoint{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            let mut s = serializer.serialize_struct("Point", 3)?;
            s.serialize_field("x", &self.point.v.x)?;
            s.serialize_field("y", &self.point.v.y)?;
            s.serialize_field("srid", &self.point.v.srid)?;
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

pub fn unique_path(prefix: &Path, extension: &Path)-> PathBuf{
    let random_str: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    
    let new_path = Path::new(&random_str);
    let mut dest = prefix.join(new_path); 
    dest.set_extension(&extension);
    if !dest.exists() {
        return dest;      
    } else {
        return unique_path(prefix, extension);
    }
}