use rocket::fairing::AdHoc;
use crate::files::intro_screen::{get_intro_image_links, get_intro_image};

mod intro_screen;

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("custom resources stage", |rocket| async {
        rocket.mount(
            "/resource",
            routes![get_intro_image_links, get_intro_image],
        )
    })
}