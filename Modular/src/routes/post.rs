use super::HtmlResponse;
use crate::fairings::db::DBConnection;
use crate::models::{
    pagination::Pagination,
    post::{NewPost, Post, ShowPost},
    post_type::PostType,
    user::User,
};
use image::codecs::jpeg::JpegEncoder;
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageEncoder};
use rocket::form::Form;
use rocket::http::Status;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket_db_pools::{sqlx::Acquire, Connection};
use rocket_dyn_templates::{context, Template};
use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Deref;
use std::path::Path;

#[get("/users/<user_uuid>/posts/<uuid>", format = "text/html")]
pub async fn get_post(
    mut db: Connection<DBConnection>,
    user_uuid: &str,
    uuid: &str,
) -> HtmlResponse {
    let connection = db
        .acquire()
        .await
        .map_err(|_| Status::InternalServerError)?;
    let user = User::find(connection, user_uuid)
        .await
        .map_err(|e| e.status)?;
    let connection = db
        .acquire()
        .await
        .map_err(|_| Status::InternalServerError)?;
    let post = Post::find(connection, uuid).await.map_err(|e| e.status)?;
    if post.user_uuid != user.uuid {
        return Err(Status::InternalServerError);
    }

    let context = context! { user, post: &(post.to_show_post())};
    Ok(Template::render("posts/show", context))
}

#[get("/users/<user_uuid>/posts?<pagination>", format = "text/html")]
pub async fn get_posts(
    mut db: Connection<DBConnection>,
    flash: Option<FlashMessage<'_>>,
    user_uuid: &str,
    pagination: Option<Pagination>,
) -> HtmlResponse {
    let flash_message = flash.map(|fm| String::from(fm.message()));
    let user = User::find(&mut db, user_uuid).await.map_err(|e| e.status)?;
    let (posts, new_pagination) = Post::find_all(&mut db, user_uuid, pagination)
        .await
        .map_err(|e| e.status)?;

    let show_posts: Vec<ShowPost> = posts.into_iter().map(|post| post.to_show_post()).collect();
    let context = context! {
    flash: flash_message,
    user,
    posts: &show_posts,
    pagination: new_pagination.map(|pg|pg.to_context())};
    Ok(Template::render("posts/index", context))
}

#[post(
"/users/<user_uuid>/posts",
format = "multipart/form-data",
data = "<upload>",
rank = 1
)]
pub async fn create_post<'r>(
    mut db: Connection<DBConnection>,
    user_uuid: &str,
    mut upload: Form<NewPost<'r>>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let create_err = || {
        Flash::error(
            Redirect::to(format!("/users/{}/posts", user_uuid)),
            "Something went wrong when uploading file",
        )
    };
    let file_uuid = uuid::Uuid::new_v4().to_string();
    if upload.file.content_type().is_none() {
        return Err(create_err());
    }
    let ext = upload.file.content_type().unwrap().extension().unwrap();
    let tmp_filename = format!("/tmp/{}.{}", &file_uuid, &ext);
    upload
        .file
        .persist_to(tmp_filename)
        .await
        .map_err(|_| create_err())?;
    let mut content = String::new();
    let mut post_type = PostType::Text;
    let mt = upload.file.content_type().unwrap().deref();
    if mt.is_text() {
        let orig_path = upload.file.path().unwrap().to_string_lossy().to_string();
        let mut text_content = vec![];
        let _ = File::open(orig_path)
            .map_err(|_| create_err())?
            .read(&mut text_content)
            .map_err(|_| create_err())?;
        content.push_str(std::str::from_utf8(&text_content).unwrap());
    } else if mt.is_bmp() || mt.is_jpeg() || mt.is_png() || mt.is_gif() {
        post_type = PostType::Photo;
        let orig_path = upload.file.path().unwrap().to_string_lossy().to_string();
        let dest_filename = format!("{}.jpg", file_uuid);
        content.push_str("/assets/");
        content.push_str(&dest_filename);

        let orig_file = File::open(orig_path).map_err(|_| create_err())?;
        let file_reader = BufReader::new(orig_file);
        let image: DynamicImage = ImageReader::new(file_reader)
            .with_guessed_format()
            .map_err(|_| create_err())?
            .decode()
            .map_err(|_| create_err())?;

        let dest_path = Path::new(rocket::fs::relative!("static")).join(&dest_filename);
        let mut file_writer = File::create(dest_path).map_err(|_| create_err())?;
        JpegEncoder::new_with_quality(&mut file_writer, 75)
            .write_image(
                image.as_bytes(),
                image.width(),
                image.height(),
                image.color(),
            )
            .map_err(|_| create_err())?;
    } else if mt.is_svg() {
        post_type = PostType::Photo;
        let dest_filename = format!("{}.svg", file_uuid);
        content.push_str("/assets/");
        content.push_str(&dest_filename);

        let dest_path = Path::new(rocket::fs::relative!("static")).join(&dest_filename);
        upload
            .file
            .move_copy_to(&dest_path)
            .await
            .map_err(|_| create_err())?;
    } else {
        return Err(create_err());
    }
    let connection = db.acquire().await.map_err(|_| create_err())?;

    Post::create(connection, user_uuid, post_type, &content)
        .await
        .map_err(|_| create_err())?;
    Ok(Flash::success(
        Redirect::to(format!("/users/{}/posts", user_uuid)),
        "Successfully created post",
    ))
}

#[delete("/users/<_user_uuid>/posts/<_uuid>", format = "text/html")]
pub async fn delete_post(
    mut _db: Connection<DBConnection>,
    _user_uuid: &str,
    _uuid: &str,
) -> HtmlResponse {
    todo!("will implement later")
}