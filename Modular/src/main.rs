#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};
use chrono::{offset::Utc, DateTime};
use rocket_db_pools::{sqlx::{FromRow, PgPool}, Database};
use uuid::Uuid;

#[derive(sqlx::Type,Debug)]
#[repr(i32)]
enum UserStatus {
    Inactive = 0,
    Active = 1,
}

#[derive(Debug, FromRow)]
struct User {
    uuid: Uuid,
    username: String,
    email: String,
    password_hash: String,
    description: String,
    status: UserStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[get("/users/<_uuid>", format="text/html")]
async fn get_user(mut _db: Connection<DBConnection>, _uuid: &str) -> HtmlReponse {
    todo!("Will implement later")
}

#[launch]
async fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(DBConnection::init());
}
