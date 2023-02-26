use std::{
    error::Error,
    path::{Path, PathBuf}, fs::{File}, io::Write, backtrace::Backtrace,
};

use chrono::Local;
use diesel::sql_types::*;
use postgis::ewkb::Point;
use postgis_diesel::sql_types::*;
use postgis_diesel::*;
use rand::{distributions::Alphanumeric, Rng};
use rocket::response::Debug;
use serde::{ser::SerializeStruct, Serialize};

pub struct InsignoPoint {
    point: PointC<Point>,
}

impl From<PointC<Point>> for InsignoPoint {
    fn from(point: PointC<Point>) -> Self {
        InsignoPoint { point }
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
/*
impl<'de> Deserialize<'de> for InsignoPoint{
    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
            let d = deserializer.deserialize_struct("Point", 3)?;

        todo!()
    }
}*/

pub struct InsignoTimeStamp {
    dtm: chrono::NaiveDateTime,
}

impl From<chrono::NaiveDateTime> for InsignoTimeStamp {
    fn from(dtm: chrono::NaiveDateTime) -> Self {
        InsignoTimeStamp { dtm }
    }
}

impl Serialize for InsignoTimeStamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("DateTime", 3)?;
        s.serialize_field("dtm", &self.dtm.to_string())?;
        s.end()
    }
}

sql_function!(fn st_transform(g: Geometry, srid: Integer)-> Geometry);
sql_function!(fn st_dwithin(g1: Geometry, g2: Geometry, dist: Double) ->  Bool); // "Represents the postgis_sql distance() function"

pub fn unique_path(prefix: &Path, extension: &Path) -> PathBuf {
    loop {
        let random_str: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        let new_path = Path::new(&random_str);
        let mut dest = prefix.join(new_path);
        dest.set_extension(extension);
        if !dest.exists() {
            return dest;
        }
    }
}

pub fn to_debug<E: Error>(err: E) -> Debug<Box<dyn Error>> {
    let bt = Backtrace::force_capture();
    let mut file = File::options().append(true).create(true).open("./log").unwrap();
    let to_write =Local::now().to_string()+" "+  &err.to_string() +"\n" +&bt.to_string() + "\n";
    file.write(to_write.as_bytes()).unwrap();
    Debug(err.to_string().into())
}
pub fn str_to_debug(s: &str) -> Debug<Box<dyn Error>> {
    let bt = Backtrace::force_capture();
    let mut file = File::options().append(true).create(true).open("./log").unwrap();
    let to_write =Local::now().to_string()+" "+  s +"\n" + &bt.to_string() + "\n";;
    file.write(to_write.as_bytes()).unwrap();
    Debug(s.into())
}
