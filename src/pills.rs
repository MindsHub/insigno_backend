use diesel::{QueryDsl, RunQueryDsl};
use rocket::{serde::json::Json, Route};
use serde::{Deserialize, Serialize};

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
    sql_function!(fn random() ->  Float);// "Represents the sql RANDOM() function"

    let res: Result<Vec<Pill>, _> = connection
        .run(|c| {
            pills::table
                .order(random())
                .limit(1)
                .load(c)
        })
        .await;

    if let Ok(res) = res {
        if let Some(res) = res.into_iter().next() {
            return Json(Some(res));
        }
    }

    Json(None)
}

pub fn get_routes() -> Vec<Route> {
    routes![get_random_pill]
}
