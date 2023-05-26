#![no_main]

use libfuzzer_sys::{fuzz_target, Corpus};
use insignolib::auth::signup::SignupInfo;
fuzz_target!(|data: String| -> Corpus {
    let v: Vec<&str> = data.splitn(3, "!").collect();
    if v.len()!=3{
        return Corpus::Reject;
    }
    let _ =SignupInfo{
        name: v[0].to_string(),
        email: v[1].to_string(),
        password: v[2].to_string(),
    }.sanitize();
    Corpus::Keep
});
