use mongodb::{Client, bson};
use rocket::{
    State,
    fs::FileServer,
    http::{Cookie, CookieJar},
};
use rocket_dyn_templates::{Template, context};
use serde::Serialize;
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
    project: Option<&'r str>,
    section: Option<&'r str>,
    tags: Option<&'r str>,
    completed: Option<bool>,
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

    let tasks = db::fetch_tasks(db, &user_id, bson::Document::new()).await;

    Template::render("index", context! { tasks })
}

// fn new_habit(habit: Habit<'_>) ->

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();

    let mongo_pw =
        env::var("MONGO_INITDB_ROOT_PASSWORD").expect("Set MONGO_INITDB_ROOT_PASSWORD env");

    let mongo_user =
        env::var("MONGO_INITDB_ROOT_USERNAME").expect("Set MONGO_INITDB_ROOT_USERNAME env");

    let mongo_url = if env::var("MONGO_URL").is_ok() {
        env::var("MONGO_URL").unwrap()
    } else {
        format!("mongodb://{}:{}@127.0.0.1:27017", mongo_user, mongo_pw)
    };

    let db = Client::with_uri_str(mongo_url)
        .await
        .expect("Error connecting to mongodb")
        .database("tasks");

    rocket::build()
        // .attach(Mongo::init())
        .manage(db)
        .mount("/", routes![main_page, public::add_task, public::get_tasks])
        .mount("/api/v1", routes![private::add_task])
        .mount("/", FileServer::from("static"))
        .mount("/wave", routes![wave, hello])
        .attach(Template::fairing())
}
