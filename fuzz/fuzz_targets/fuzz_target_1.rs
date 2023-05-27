#![no_main]
use insignolib::rocket;
use libfuzzer_sys::{fuzz_target, Corpus};
use insignolib::{rocket::{http::ContentType, local::blocking::Client}};

fuzz_target!(|data: String| -> Corpus {
    let client = Client::tracked(rocket())
            .expect("valid rocket instance");
    let data = data;

    let _response = client
        .post("/signup")
        .header(ContentType::Form)
        .body(data)
        .dispatch();
    Corpus::Keep
});
