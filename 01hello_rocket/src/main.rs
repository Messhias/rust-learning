#[macro_use]
extern crate rocket;
use rocket::{Build, Rocket};

#[get("/")]
fn index() -> &'static str {
    "Hello, rocket"
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![index])
}
