#[macro_use]
extern crate rocket;

use lazy_static::lazy_static;
use rocket::http::{ContentType, Status};
use rocket::request::{FromParam, Request};
use rocket::response::{self, Responder, Response};
use rocket::{Build, Rocket, State};
use std::collections::HashMap;
use std::io::Cursor;
use std::vec::Vec;
use std::sync::atomic::{AtomicU64, Ordering};

struct VisitorCounter {
    visitor: AtomicU64,
}

#[derive(FromForm)]
struct Filters {
    age: u8,
    active: bool,
}

struct NameGrade<'r> {
    name: &'r str,
    grade: u8,
}

impl<'r> FromParam<'r> for NameGrade<'r> {
    type Error = &'static str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        const ERROR_MESSAGE: Result<NameGrade, &'static str> = Err("Error parsing user parameter");

        let name_grade_vec: Vec<&'r str> = param.split('_').collect();
        match name_grade_vec.len() {
            2 => match name_grade_vec[1].parse::<u8>() {
                Ok(n) => Ok(Self {
                    name: name_grade_vec[0],
                    grade: n,
                }),
                Err(_) => ERROR_MESSAGE,
            },
            _ => ERROR_MESSAGE,
        }
    }
}

#[derive(Debug)]
struct User {
    uuid: String,
    name: String,
    age: u8,
    grade: u8,
    active: bool,
}

impl<'r> Responder<'r, 'r> for &'r User {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'r> {
        let user = format!("Found user: {:?}", self);
        Response::build()
            .sized_body(user.len(), Cursor::new(user))
            .raw_header("X-USER-ID", self.uuid.to_string())
            .header(ContentType::Plain)
            .ok()
    }
}

struct NewUser<'a>(Vec<&'a User>);

impl<'r> Responder<'r, 'r> for NewUser<'r> {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'r> {
        let user = self
            .0
            .iter()
            .map(|u| format!("{:?}", u))
            .collect::<Vec<String>>()
            .join(",");
        Response::build()
            .sized_body(user.len(), Cursor::new(user))
            .header(ContentType::Plain)
            .ok()
    }
}

lazy_static! {
    static ref USERS: HashMap<&'static str, User> = {
        let mut map = HashMap::new();
        map.insert(
            "3e3dd4ae-3c37-40c6-aa64-7061f284ce28",
            User {
                uuid: String::from("3e3dd4ae-3c37-40c6-aa64-7061f284ce28"),
                name: String::from("John Doe"),
                age: 18,
                grade: 1,
                active: true,
            },
        );
        map.insert(
            "abc123",
            User {
                uuid: String::from("acb123"),
                name: String::from("Fabio"),
                age: 33,
                grade: 2,
                active: true,
            },
        );
        map
    };
}

impl VisitorCounter {
    fn increment_counter(&self) {
        self.visitor.fetch_add(1, Ordering::Relaxed);
        println!(
            "The number of visitor is: {}",
            self.visitor.load(Ordering::Relaxed)
        );
    }
}

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("We cannot find this page {}.", req.uri())
}

#[catch(403)]
fn forbidden(req: &Request) -> String {
    format!("Access forbidden {}.", req.uri())
}

#[get("/user/<uuid>", rank = 1, format = "text/plain")]
fn user<'a>(counter: &State<VisitorCounter>, uuid: &str) -> Option<&'a User> {
    counter.increment_counter();
    USERS.get(uuid)
}

#[get("/users/<name_grade>?<filters..>")]
fn users<'a>(
    counter: &State<VisitorCounter>,
    name_grade: NameGrade,
    filters: Option<Filters>,
) -> Result<NewUser<'a>, Status> {
    counter.increment_counter();
    let users: Vec<&User> = USERS
        .values()
        .filter(|user| user.name.contains(&name_grade.name) && user.grade == name_grade.grade)
        .filter(|user| {
            if let Some(fts) = &filters {
                user.age == fts.age && user.active == fts.active
            } else {
                true
            }
        })
        .collect();
    if users.is_empty() {
        Err(Status::Forbidden)
    } else {
        Ok(NewUser(users))
    }
}

#[launch]
fn rocket() -> Rocket<Build> {
    let visitor_counter = VisitorCounter {
        visitor: AtomicU64::new(0),
    };
    rocket::build()
        .manage(visitor_counter)
        .mount("/", routes![user, users])
        .register("/", catchers![not_found, forbidden])
}