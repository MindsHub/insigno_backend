use postgis_diesel::operators::*;
use postgis_diesel::types::*;
use rocket::Route;
table! {
    marker (id) {
        id -> Int4,
        point-> postgis_diesel::sql_types::Geometry,
    }
}
#[derive(Clone, Queryable, Debug, Insertable)]
#[diesel(table_name = marker)]

struct Marker {
    id: i32,
    point: Point,
}

#[get("/get_near")]
pub fn get_near() {
    let z = Point {
        x: 0.0,
        y: 0.0,
        srid: Some(5123),
    };
}

pub fn get_routes() -> Vec<Route> {
    routes![get_near]
}
