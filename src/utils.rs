use std::{
    backtrace::Backtrace,
    error::Error,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::Local;

use rand::{distributions::Alphanumeric, Rng};
use rocket::response::Debug;

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
