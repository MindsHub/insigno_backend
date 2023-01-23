use diesel::{RunQueryDsl, QueryDsl};
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
    // this allows executing this query: SELECT * FROM pills ORDER BY RANDOM() LIMIT 1
    no_arg_sql_function!(RANDOM, (), "Represents the sql RANDOM() function");

    let res: Result<Vec<Pill>, _> = connection
        .run(|c| pills::table
            .order(RANDOM)
            .limit(0)
            .load(c))
        .await;

    if let Ok(res) = res {
        if let Some(res) = res.into_iter().nth(0) {
            return Json(Some(res));
        }
    }

    return Json(None);
}

pub fn get_routes() -> Vec<Route>{
    routes![get_random_pill]
}