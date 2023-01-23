use diesel::RunQueryDsl;
use rand::Rng;
use rocket::{serde::json::Json, Route};
use serde::{Serialize, Deserialize};

use super::db::Db;

table! {
    pills (id) {
        id -> Int4,
        text -> Text,
        author -> Text,
        source -> Text,
    }
}

#[derive(Serialize, Deserialize, Clone, Queryable, Debug, Insertable)]
#[table_name = "pills"]
struct Pill {
    id: i32,
    text: String,
    author: String,
    source: String,
}

#[get("/random")]
async fn get_random_pill(connection: Db) -> Json<Option<Pill>> {
    let res: Result<Json<Vec<Pill>>, _> = connection
        .run(|c| pills::table.load(c))
        .await
        .map(Json);
    if let Ok(Json(res)) = res{
        let mut rng = rand::thread_rng();
        let pos=rng.gen_range(0..res.len());
        Json(Some(res[pos].clone()))
    }else{
        Json(None)
    }
}

pub fn get_routes() -> Vec<Route>{
    routes![get_random_pill]
}