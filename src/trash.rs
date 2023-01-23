use rocket::Route;

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    geometry_samples (id) {
        id -> Int4,
        point -> Geometry,
        linestring -> Geometry,
    }
}

#[get("/get_near")]
pub fn get_near(){

}

pub fn get_routes() -> Vec<Route>{
    routes![get_near]
}