use crate::auth::user::{Authenticated, User};
use crate::schema_sql::pills;
use crate::utils::InsignoError;
use diesel::ExpressionMethods;
use diesel::{insert_into, QueryDsl, RunQueryDsl};
use rocket::{form::Form, serde::json::Json, Route};
use serde::{Deserialize, Serialize};

use super::db::Db;

#[derive(Serialize, Deserialize, Clone, Queryable, Debug, Insertable)]
#[diesel(table_name = pills)]
struct Pill {
    #[diesel(deserialize_as = i64)]
    id: Option<i64>,
    text: String,
    author: String,
    source: String,
    accepted: bool,
}

sql_function!(fn random()-> Double); // "Represents the sql RANDOM() function"

#[get("/random")]
async fn get_random_pill(connection: Db) -> Result<Option<Json<Pill>>, InsignoError> {
    // this allows executing this query: SELECT * FROM pills ORDER BY RANDOM() LIMIT 1

    let res: Vec<Pill> = connection
        .run(|c| {
            pills::table
                .filter(pills::accepted.eq(true))
                .order(random())
                .limit(1)
                .load(c)
        })
        .await
        .map_err(|e| InsignoError::new(404, "impossible to obtain pill", &e.to_string()))?;

    let pill = res.into_iter().next();
    if let Some(p) = pill {
        Ok(Some(Json(p)))
    } else {
        Ok(None)
    }
}

#[derive(Deserialize, Serialize, FromForm)]
struct AddPill {
    text: String,
    source: String,
}

#[post("/add", data = "<data>")]
async fn add_pill(
    connection: Db,
    data: Form<AddPill>,
    user: User<Authenticated>,
) -> Result<String, InsignoError> {
    let pill = Pill {
        id: None,
        text: data.text.clone(),
        author: user.name.clone(),
        source: data.source.clone(),
        accepted: false,
    };
    connection
        .run(move |conn| {
            use pills::dsl::pills as pi;
            insert_into(pi).values(&pill).execute(conn)
        })
        .await
        .map_err(|e| InsignoError::new(500, "impossible to add pill", &e.to_string()))?;

    Ok("Added".to_string())
}

pub fn get_routes() -> Vec<Route> {
    routes![get_random_pill, add_pill]
}

#[cfg(test)]
mod test {
    use crate::db::Db;
    use crate::diesel::ExpressionMethods;
    use crate::diesel::RunQueryDsl;
    use crate::pills::AddPill;
    use crate::rocket;
    use crate::test::*;
    use rocket::http::{ContentType, Status};
    use rocket::serde::json::serde_json;

    #[rocket::async_test]
    async fn test_pills() {
        test_reset_db();
        use rocket::local::asynchronous::Client;
        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");

        //erase_tables!(client, user_sessions, users, pills);
        // try to get a pill with an empty database
        let response = client.get("/pills/random").dispatch();
        assert_eq!(response.await.status(), Status::NotFound);

        // unautenticate add
        let new_pill = AddPill {
            text: "test".to_string(),
            source: "test".to_string(),
        };
        let input: String = serde_json::to_string(&new_pill).unwrap();
        let response = client
            .post("/pills/add")
            .header(ContentType::JSON)
            .body(input.clone())
            .dispatch();
        assert_eq!(response.await.status(), Status::Unauthorized);

        //signup
        test_signup(&client).await;

        let input = "text=test&source=test";
        // add
        let response = client
            .post("/pills/add")
            .header(ContentType::Form)
            .body(input)
            .dispatch();
        assert_eq!(response.await.status(), Status::Ok);

        // try to get a pill with a pill not reviewed in Database
        let response = client.get("/pills/random").dispatch();
        assert_eq!(response.await.status(), Status::NotFound);

        //update pill state
        let connection = &Db::get_one(client.rocket()).await.unwrap();
        let rows = connection
            .run(|c| {
                use crate::schema_sql::pills::dsl::*;
                diesel::update(pills).set(accepted.eq(true)).execute(c)
            })
            .await
            .expect("unable to modify db");
        if rows != 1 {
            panic!("row not modified");
        }

        let response = client.get("/pills/random").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            format!(
                r#"{{"id":1,"text":"test","author":"IlMagicoTester","source":"test","accepted":true}}"#
            ),
            response.into_string().await.unwrap()
        );
    }
}
