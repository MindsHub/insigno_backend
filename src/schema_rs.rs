use chrono::Utc;
use postgis::ewkb::Point;
use postgis_diesel::PointC;
use rocket::serde::Serialize;
use serde::ser::SerializeStruct;

use crate::schema_sql::*;
use crate::utils::*;

#[derive(Serialize, Clone, Queryable, Debug)]
#[diesel(table_name = "trash_types")]
pub struct TrashType {
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
    pub trash_type_id: i64,
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
        s.serialize_field("trash_type_id", &self.trash_type_id)?;
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
