use std::error::Error;

use crate::schema_sql::pills;
use crate::utils::to_debug;
use diesel::ExpressionMethods;
use diesel::{insert_into, sql_types::Double, QueryDsl, RunQueryDsl};
use rocket::response::Debug;
use rocket::{serde::json::Json, Route};
use rocket_auth::User;
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
async fn get_random_pill(connection: Db) -> Result<Option<Json<Pill>>, Debug<Box<dyn Error>>> {
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

    let pill = res.into_iter().next();
    if let Some(p) = pill {
        Ok(Some(Json(p)))
    } else {
        Ok(None)
    }
}

#[derive(Deserialize, Serialize)]
struct AddPill {
    text: String,
    source: String,
}

#[post("/add", data = "<data>")]
async fn add_pill(
    connection: Db,
    data: Json<AddPill>,
    user: User,
) -> Result<String, Debug<Box<dyn Error>>> {
    let pill = Pill {
        id: None,
        text: data.text.clone(),
        author: user.email,
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

#[cfg(test)]
mod test {
    use std::process::*;

    use crate::db::Db;
    use crate::diesel::ExpressionMethods;
    use crate::diesel::RunQueryDsl;
    use crate::pills::AddPill;
    use crate::rocket;
    use rocket::http::{ContentType, Status};
    use rocket::serde::json::serde_json;

    #[rocket::async_test]
    async fn test_pills() {
        let success = Command::new("diesel")
            .args(["database", "reset"])
            .output()
            .unwrap()
            .status
            .success();
        assert!(success);
        use rocket::local::asynchronous::Client;
        let client = Client::tracked(rocket())
            .await
            .expect("valid rocket instance");

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
        let data = "email=test@gmail.com&password=Testtes1";
        let response = client
            .post("/signup")
            .header(ContentType::Form)
            .body(data)
            .dispatch();
        assert_eq!(response.await.status(), Status::Ok);

        // add
        let response = client
            .post("/pills/add")
            .header(ContentType::JSON)
            .body(input)
            .dispatch();
        assert_eq!(response.await.status(), Status::Ok);

        // try to get a pill with a pill not reviewed in Database
        let response = client.get("/pills/random").dispatch();
        assert_eq!(response.await.status(), Status::NotFound);

        //update pill state
        //println!("{:?}", client.rocket().
        let connection = &Db::get_one(client.rocket()).await.unwrap();
        //let conn = PgConnection::establish(&config.url).unwrap();

        //let connection = client.rocket().state::<Db>().unwrap();
        let y = connection
            .run(|c| {
                use crate::schema_sql::pills::dsl::*;
                diesel::update(pills).set(accepted.eq(true)).execute(c)
            })
            .await
            .expect("unable to modify db");
        if y != 1 {
            panic!("row not modified");
        }

        let response = client.get("/pills/random").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            r#"{"id":1,"text":"test","author":"test@gmail.com","source":"test","accepted":true}"#,
            response.into_string().await.unwrap()
        );
        //
        //assert_eq!(response.into_string().unwrap(), "Hello, world!");
    }
}
