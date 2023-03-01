use std::error::Error;

use crate::schema_sql::pills;
use crate::utils::{str_to_debug, to_debug};
use diesel::ExpressionMethods;
use diesel::{insert_into, sql_types::Double, QueryDsl, RunQueryDsl};
use rocket::response::Debug;
use rocket::{serde::json::Json, Route};
use serde::{Deserialize, Serialize};

use super::db::Db;

#[derive(Serialize, Deserialize, Clone, Queryable, Debug, Insertable)]
#[diesel(table_name = pills)]
struct Pill {
    #[diesel(deserialize_as = "i64")]
    id: Option<i64>,
    text: String,
    author: String,
    source: String,
    accepted: bool,
}

no_arg_sql_function!(random, Double, "Represents the sql RANDOM() function"); // "Represents the sql RANDOM() function"

#[get("/random")]
async fn get_random_pill(connection: Db) -> Result<Json<Pill>, Debug<Box<dyn Error>>> {
    // this allows executing this query: SELECT * FROM pills ORDER BY RANDOM() LIMIT 1

    let res: Vec<Pill> = connection
        .run(|c| {
            pills::table
                .filter(pills::accepted.eq(true))
                .order(random)
                .limit(1)
                .load(c)
        })
        .await
        .map_err(to_debug)?;

    let pill = res
        .into_iter()
        .next()
        .ok_or(str_to_debug("returned 0 pills"))?;
    Ok(Json(pill))
}

#[derive(Deserialize)]
struct AddPill {
    text: String,
    author: String,
    source: String,
}

#[post("/add", data = "<data>")]
async fn add_pill(connection: Db, data: Json<AddPill>) -> Result<String, Debug<Box<dyn Error>>> {
    let pill = Pill {
        id: None,
        text: data.text.clone(),
        author: data.author.clone(),
        source: data.source.clone(),
        accepted: false,
    };
    connection
        .run(move |conn| {
            use pills::dsl::pills as pi;
            insert_into(pi).values(&pill).execute(conn)
        })
        .await
        .map_err(to_debug)?;

    Ok("Added".to_string())
}

pub fn get_routes() -> Vec<Route> {
    routes![get_random_pill, add_pill]
}
