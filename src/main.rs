use mongodb::{Client, bson};
use rocket::{
    State,
    fs::FileServer,
    http::{Cookie, CookieJar},
};
use rocket_dyn_templates::{Template, context};
use serde::Serialize;
use std::fs;
use std::vec;
use uuid::Uuid;

mod api;
mod db;
mod models;

use crate::api::{private, public};

#[macro_use]
extern crate rocket;

// #[cfg(test)]
// mod tests;

#[derive(FromFormField)]
enum Lang {
    #[field(value = "en")]
    English,
    #[field(value = "ru")]
    #[field(value = "ру")]
    Russian,
}

#[derive(FromForm)]
struct Options<'r> {
    emoji: bool,
    name: Option<&'r str>,
}

#[derive(FromForm, rocket::serde::Deserialize, rocket::serde::Serialize)]
struct TaskOptions<'r> {
    name: &'r str,
    description: Option<&'r str>,
    due: Option<u32>,
    project: &'r str,
    section: Option<&'r str>,
    tags: Option<&'r str>,
    completed: Option<bool>,
    priority: Option<models::TaskPriority>,
}

// Try visiting:
//   http://127.0.0.1:8000/wave/Rocketeer/100
#[get("/<name>/<age>", rank = 2)]
fn wave(name: &str, age: u8) -> String {
    format!("👋 Hello, {} year old named {}!", age, name)
}

// Note: without the `..` in `opt..`, we'd need to pass `opt.emoji`, `opt.name`.
//
// Try visiting:
//   http://127.0.0.1:8000/?emoji
//   http://127.0.0.1:8000/?name=Rocketeer
//   http://127.0.0.1:8000/?lang=ру
//   http://127.0.0.1:8000/?lang=ру&emoji
//   http://127.0.0.1:8000/?emoji&lang=en
//   http://127.0.0.1:8000/?name=Rocketeer&lang=en
//   http://127.0.0.1:8000/?emoji&name=Rocketeer
//   http://127.0.0.1:8000/?name=Rocketeer&lang=en&emoji
//   http://127.0.0.1:8000/?lang=ru&emoji&name=Rocketeer
#[get("/?<lang>&<opt..>")]
fn hello(lang: Option<Lang>, opt: Options<'_>) -> String {
    let mut greeting = String::new();
    if opt.emoji {
        greeting.push_str("👋 ");
    }

    match lang {
        Some(Lang::Russian) => greeting.push_str("Привет"),
        Some(Lang::English) => greeting.push_str("Hello"),
        None => greeting.push_str("Hi"),
    }

    if let Some(name) = opt.name {
        greeting.push_str(", ");
        greeting.push_str(name);
    }

    greeting.push('!');
    greeting
}

#[derive(Serialize)]
struct AddTaskResponse {
    success: bool,
    message: String,
    task_id: String,
}

// render main tracker
#[get("/")]
async fn main_page(cookies: &CookieJar<'_>, db: &State<mongodb::Database>) -> Template {
    let user_id = match cookies.get("uuid") {
        Some(c) => c.value().to_string(),
        None => {
            let uuid = Uuid::new_v4().to_string();
            cookies.add(Cookie::build(("uuid", uuid.clone())).path("/").permanent());
            uuid
        }
    };

    let tasks = db::fetch_tasks(db, bson::doc! { "completed": false, "user_id": &user_id }).await;
    let projects = db::get_unique_projects(db, &user_id)
        .await
        .unwrap_or_default();

    let css_file = fs::read_to_string("static/style.css").unwrap_or_default();

    let css_hash = seahash::hash(css_file.as_bytes());

    Template::render("index", context! { tasks, projects, css_hash })
}

// fn new_habit(habit: Habit<'_>) ->

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();

    let mongo_pw =
        env::var("MONGO_INITDB_ROOT_PASSWORD").expect("Set MONGO_INITDB_ROOT_PASSWORD env");

    let mongo_user =
        env::var("MONGO_INITDB_ROOT_USERNAME").expect("Set MONGO_INITDB_ROOT_USERNAME env");

    let mongo_url = if let Ok(url) = env::var("MONGO_URL") {
        url
    } else {
        format!("mongodb://{}:{}@127.0.0.1:27017", mongo_user, mongo_pw)
    };

    let app_name = "focus_passion";

    let db_name = if env::var("FOCUS_PASSION_DEV").is_ok() {
        app_name.to_string()
    } else {
        format!("{}_prod", app_name)
    };

    let db = Client::with_uri_str(mongo_url)
        .await
        .expect("Error connecting to mongodb")
        .database(&db_name);

    rocket::build()
        // .attach(Mongo::init())
        .manage(db)
        .mount("/", routes![main_page,])
        .mount("/tasks", public::routes())
        .mount("/api/v1", routes![private::add_task])
        .mount("/", FileServer::from("static"))
        .mount("/wave", routes![wave, hello])
        .attach(Template::fairing())
}
