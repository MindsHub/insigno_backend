use diesel::{QueryDsl, RunQueryDsl, sql_types::Double};
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
#[diesel(table_name = pills)]
struct Pill {
    id: i32,
    text: String,
    author: String,
    source: String,
}

no_arg_sql_function!(random, Double, "Represents the sql RANDOM() function"); // "Represents the sql RANDOM() function"
#[get("/random")]
async fn get_random_pill(connection: Db) -> Json<Option<Pill>> {
    // this allows executing this query: SELECT * FROM pills ORDER BY RANDOM() LIMIT 1
    

    let res: Result<Vec<Pill>, _> = connection
        .run(|c| pills::table.order(random).limit(1).load(c))
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
