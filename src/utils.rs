use std::{
    backtrace::Backtrace,
    error::Error,
    fs::File,
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

use chrono::Local;

use rand::{distributions::Alphanumeric, Rng};
use rocket::response::Responder;
use rocket::{
    http::Status,
    response::{self, Debug},
    Request, Response,
};

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

pub fn to_debug<E: Error>(err: E) -> Debug<Box<dyn Error>> {
    let bt = Backtrace::force_capture();
    let mut file = File::options()
        .append(true)
        .create(true)
        .open("./log")
        .unwrap();
    let to_write =
        Local::now().to_string() + " " + &err.to_string() + "\n" + &bt.to_string() + "\n";
    file.write_all(to_write.as_bytes()).unwrap();
    Debug(err.to_string().into())
}

pub fn str_to_debug(s: &str) -> Debug<Box<dyn Error>> {
    let bt = Backtrace::force_capture();
    let mut file = File::options()
        .append(true)
        .create(true)
        .open("./log")
        .unwrap();
    let to_write = Local::now().to_string() + " " + s + "\n" + &bt.to_string() + "\n";
    file.write_all(to_write.as_bytes()).unwrap();
    Debug(s.into())
}

pub struct InsignoError {
    debug: Option<String>,
    client: Option<String>,
    code: Status,
}

#[allow(dead_code)]
impl InsignoError {
    pub fn new_code(v: i32) -> Self {
        InsignoError {
            debug: None,
            client: None,
            code: Status { code: v as u16 },
        }
    }
    pub fn new_debug(v: i32, s: &str) -> Self {
        InsignoError {
            debug: Some(s.to_string()),
            client: None,
            code: Status { code: v as u16 },
        }
    }
    pub fn new(v: i32, client: &str, debug: &str) -> Self {
        InsignoError {
            debug: Some(debug.to_string()),
            client: Some(client.to_string()),
            code: Status { code: v as u16 },
        }
    }
}

impl<'r> Responder<'r, 'static> for InsignoError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let deb_str = self.debug.unwrap_or(
            self.client
                .clone()
                .unwrap_or("no string provided".to_string()),
        );
        let bt = Backtrace::force_capture();
        let mut file = File::options()
            .append(true)
            .create(true)
            .open("./log")
            .unwrap();
        let to_write = Local::now().to_string() + " " + &deb_str + "\n" + &bt.to_string() + "\n";
        file.write_all(to_write.as_bytes()).unwrap();

        if let Some(v) = self.client {
            Response::build_from(self.code.respond_to(req)?)
                .sized_body(v.len(), Cursor::new(v))
                .ok()
        } else {
            Response::build_from(self.code.respond_to(req)?).ok()
        }
        /*Response::build_from(string.respond_to(req)?)
        //.raw_header("X-Person-Name", self.name)
        //.raw_header("X-Person-Age", self.age.to_string())
        //.header(ContentType::new("application", "x-person"))
        .status(self.code)
        .ok()*/
    }
}
