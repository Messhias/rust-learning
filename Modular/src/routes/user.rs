use super::HtmlResponse;
use crate::fairings::db::DBConnection;
use crate::models::{pagination::Pagination, user::User};
use rocket::form::Form;
use rocket_db_pools::{Connection, sqlx::Acquire};
use rocket::http::Status;
use rocket::response::content::RawHtml;

const USER_HTML_PREFIX: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title> Our Application User </title>
</head>
<body>"#;

const USER_HTML_SUFFIX: &str = r#"</body></html>"#;

#[get("/users/<_uuid>", format = "text/html")]
pub async fn get_user(mut db: Connection<DBConnection>, uuid: &str) -> HtmlResponse {
    let connection = db.acquire()
        .await
        .map_err(|_| Status::InternalServerError)?;
    let user = User::find(connection, uuid)
        .await
        .map(|_| Status::NotFound)?;
    let mut html_string = String::from(USER_HTML_PREFIX);
    html_string.push_str(
        format!(r#"
            <a href="/users/edit/{}">Edit user</a>
        "#, user.uuid
        ).as_ref(),
    );

    html_string.push_str(r#"
        <a href="/users">User list</a>
    "#);

    html_string.push_str(USER_HTML_SUFFIX);

    Ok(RawHtml(html_string))
}

#[get("/users?<_pagination>", format = "text/html")]
pub async fn get_users(
    mut db: Connection<DBConnection>,
    pagination: Option<Pagination>,
) -> HtmlResponse {
    let (users, new_pagination) = User::find_all(&mut db, pagination)
        .await
        .map_err(|_| Status::NotFound)?;
    let mut html_string = String::from(USER_HTML_PREFIX);

    for user in users {
        html_string.push_str(&user.to_html_string());
        html_string.push_str(
            format!(r#"
                <a href="users/{}">See user<a/><br/>
            "#, user.uuid).as_ref()
        );
        html_string.push_str(
            format!(r#"
                <a href="/users/edit/{}">Edit user</a><br/>
            "#, user.uuid).as_ref(),
        );

        if let Some(pg) = new_pagination {
            html_string.push_str(
                format!(r#"
                    <a href="/users?pagination.next={}&pagination.limit={}">Next</a></br>
                "#,
                    &(pg.next.0).timestamp_nanos(),
                    &pg.limit,
                ).as_ref(),
            );
        }
    }

    html_string.push_str(r#"
        <a href="/users/new">New user</a>
    "#);
    html_string.push_str(USER_HTML_SUFFIX);

    Ok(RawHtml(html_string))
}

#[get("/users/new", format = "text/html")]
pub async fn new_user(mut _db: Connection<DBConnection>) -> HtmlResponse {
    let mut html_string = String::from(USER_HTML_PREFIX);
    html_string.push_str(r#"
        <form accept-chartset="UTF-8" action="/users"
            autocomplete="off" method="POST">
            <div>
                <label for="username">Username:</label>
                <input name="username" type="text"/>
            </div>
            <div>
                <label for="username">Username:</label>
                <input name="username" type="text"/>
            </div>
        </form>
    "#);
}

#[post("/users", format = "text/html", data = "<_user>")]
pub async fn create_user(mut _db:
                         Connection<DBConnection>, _user: Form<User>) ->
                         HtmlResponse {
    todo!("will implement later")
}

#[get("/users/edit/<_uuid>", format = "text/html")]
pub async fn edit_user(mut _db: Connection<DBConnection>,
                       _uuid: &str) -> HtmlResponse {
    todo!("will implement later")
}

#[put("/users/<uuid>", format = "text/html", data = "<_user>")]
pub async fn put_user(
    mut _db: Connection<DBConnection>,
    _uuid: &str,
    _user: Form<User>,
) -> HtmlResponse {
    todo!("will implement later")
}

#[patch("/users/<_uuid>", format = "text/html", data = "<_user>")]
pub async fn patch_user(
    mut _db: Connection<DBConnection>,
    _uuid: &str,
    _user: Form<User>,
) -> HtmlResponse {
    todo!("will implement later")
}

#[delete("/users/<_uuid>", format = "text/html")]
pub async fn delete_user(mut _db:
                         Connection<DBConnection>, _uuid: &str) -> HtmlResponse {
    todo!("will implement later")
}