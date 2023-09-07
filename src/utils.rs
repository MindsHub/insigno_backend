use std::{
    backtrace::Backtrace,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::Local;

use rand::{distributions::Alphanumeric, Rng};
use rocket::{
    http::Status,
    response::{self},
    Request,
};
use rocket::{request, response::Responder};

pub fn unique_path(prefix: &Path, extension: &Path) -> PathBuf {
    loop {
        let random_str: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        let new_path = Path::new(&random_str);
        let mut dest = prefix.join(new_path);
        dest.set_extension(extension);
        if !dest.exists() {
            return dest;
        }
    }
}

#[derive(Debug)]
pub struct InsignoError {
    debug: Option<String>,
    client: Option<String>,
    code: Status,
}

#[allow(dead_code)]
impl InsignoError {
    pub fn client<T: ToString>(mut self, s: T) -> Self {
        self.client = Some(s.to_string());
        self
    }
    pub fn debug<T: ToString>(mut self, s: T) -> Self {
        let s = s.to_string();
        #[cfg(test)]
        {
            let bt = Backtrace::force_capture();
            println!("{s}\n{bt}");
        }
        self.debug = Some(s);
        self
    }
    pub fn both<T: ToString>(mut self, s: T) -> Self {
        self.debug = Some(s.to_string());
        self.client = Some(s.to_string());
        self
    }

    pub fn new(v: i32) -> Self {
        InsignoError {
            debug: None,
            client: None,
            code: Status { code: v as u16 },
        }
    }
}

impl<T> From<InsignoError> for request::Outcome<T, InsignoError> {
    fn from(val: InsignoError) -> Self {
        request::Outcome::Failure((val.code, val))
    }
}

impl<'r> Responder<'r, 'static> for InsignoError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        if let Some(s) = self.debug {
            let bt = Backtrace::force_capture();
            let mut file = File::options()
                .append(true)
                .create(true)
                .open("./log")
                .unwrap();
            let to_write = Local::now().to_string() + " " + &s + "\n" + &bt.to_string() + "\n";

            file.write_all(to_write.as_bytes()).unwrap();
        }

        use rocket::response::{content, status};
        if let Some(v) = self.client {
            status::Custom(self.code, content::RawText(v)).respond_to(req)
        } else {
            self.code.respond_to(req)
        }
    }
}

#[macro_export]
macro_rules! erase_tables {
    ( $client:expr, $( $table:ident ),* ) => {
        {
            let connection = Db::get_one($client.rocket()).await.unwrap();
            connection.run(|conn| {
                $(
                    diesel::delete($crate::schema_sql::$table::dsl::$table).execute(conn).unwrap();
                )*
                println!("test");
            }).await
        }
    };
}
