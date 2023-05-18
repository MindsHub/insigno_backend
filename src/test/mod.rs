use std::process::Command;

use rocket::{
    figment::providers::{Format, Toml},
    http::{ContentType, Status},
    local::asynchronous::Client,
    Config,
};

pub fn test_reset_db() {
    println!("cleaning db");
    let figment = Config::figment().merge(Toml::file("Insigno.toml").nested());
    let value = figment.find_value("databases.db.url").unwrap();
    println!("{value:?}");
    let url = value.as_str().unwrap();

    let output = Command::new("diesel")
        .args(["database", "reset", &format!("--database-url={url}")])
        .output()
        .unwrap();

    //println!("{}", );
    assert!(
        output.status.success(),
        "{:?}",
        String::from_utf8(output.stderr)
    );
}

#[cfg(test)]
pub async fn test_signup(client: &Client) -> i64 {
    let data = "name=IlMagicoTester&password=Testtes1!&email=test@test.com";
    let response = client
        .post("/signup")
        .header(ContentType::Form)
        .body(data)
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);
    let text = response.into_string().await.unwrap();
    println!("{}", text);

    let response = client.get("/verify/1111111111").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let data = "password=Testtes1!&email=test@test.com";
    let response = client
        .post("/login")
        .header(ContentType::Form)
        .body(data)
        .dispatch()
        .await;
    //let y = response.body();
    let text = response.into_string().await.unwrap();
    println!("{}", text);
    //assert_eq!(response.status(), Status::Ok);
    //response.into_string().await.unwrap().parse::<i64>().unwrap()
    text.parse::<i64>().unwrap()
}

pub async fn test_add_point(client: &Client) {
    let response = client
        .post("/map/add")
        .header(ContentType::Form)
        .body("x=0.0&y=0.0&marker_types_id=2")
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);
    /*assert_eq!(
        response.into_string().await.unwrap(),
        format!("{{\"id\":{id},\"earned_points\":1.0}}")
    );*/
}

pub async fn test_add_image(marker_id: i64, path: &str, c: &Client) {
    use form_data_builder::FormData;
    let mut form = FormData::new(Vec::new()); // use a Vec<u8> as a writer;

    form.write_path("image", path, "image/jpg").unwrap();
    form.write_field("refers_to_id", &marker_id.to_string())
        .unwrap();
    let y = form.finish().unwrap(); // returns the writer
    let temp_str = form.content_type_header();
    let w: Vec<&str> = temp_str.split('/').collect();

    let response = c
        .post("/map/image/add")
        .header(ContentType::new(w[0].to_string(), w[1].to_string()))
        .body(y)
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);
}

struct InsignoTest {
    f: Box<dyn Fn()>,
}

impl Drop for InsignoTest {
    fn drop(&mut self) {
        (self.f)()
    }
}
/*
#[macro_export]
macro_rules! clean_db {
    (d $db:expr, $exp:ident) => {
        println!("{}", stringify!($exp));
        //sql_query(format!("ALTER SEQUENCE {}_id_seq RESTART WITH 1", stringify!($exp))).execute($db).unwrap();
        let _y = delete($exp::dsl::$exp).execute($db).unwrap();
    };
    (d $db:expr, $exp:ident, $( $x:ident ),* ) => {
        {
            clean_db!(d $db, $exp);
            clean_db!(d $db, $($x),*);
        }
    };
    ( $( $x:ident ),+ ) => {
        use diesel::{PgConnection, Connection, RunQueryDsl};
        use diesel::result::Error;
        use diesel::sql_query;
        use diesel::delete;
        use rocket::Rocket;
        let value = Rocket::build()
            .figment()
            .find_value("databases.db.url")
            .unwrap();
        let url = value
            .as_str()
            .unwrap();

        let mut conn = PgConnection::establish(&url).unwrap();
        conn.transaction::<_, Error, _>(|conn| {
            clean_db!(d conn, $($x),+);
            Ok(())
        }).unwrap();
    }
}*/

#[cfg(test)]
mod test {
    use diesel::{Connection, PgConnection};
    use rocket::{
        figment::providers::{Format, Toml},
        Config,
    };

    //use crate::schema_sql::users;
    #[test]
    fn test1() {
        let figment = Config::figment().merge(Toml::file("Insigno.toml").nested());
        let value = figment.find_value("databases.db.url").unwrap();
        let url = value.as_str().unwrap();

        let _conn = PgConnection::establish(url).unwrap();
        //conn.transaction(f)
        //let  y = users::all_columns.0;
        //let y: i64 = y.assume_not_null().into();
    }
}
