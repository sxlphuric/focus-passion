use mongodb::{Client, bson};
use nanoid::nanoid;
use rocket::{State, fs::FileServer, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use serde::Serialize;
use std::vec;

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

#[derive(FromForm)]
struct TaskOptions<'r> {
    name: &'r str,
    description: Option<&'r str>,
    due: Option<u32>,
    project: Option<&'r str>,
    tags: Option<&'r str>,
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

#[get("/?<id>&<opt..>")]
#[allow(unused_mut)]
async fn add_task(
    id: String,
    opt: TaskOptions<'_>,
    db: &State<mongodb::Database>,
) -> Json<AddTaskResponse> {
    let collection = db.collection::<bson::Document>(&id);

    let task_id: String = nanoid!();

    let mut task = bson::Document::new();

    task.insert("id", &task_id);
    task.insert("name", opt.name);
    task.insert("description", opt.description);
    task.insert("due", opt.due);
    task.insert("project", opt.project);
    task.insert("completed", false);

    if let Some(tags) = opt.tags {
        let tags_string = String::from(tags);
        let tags_vec: Vec<&'_ str> = tags_string.split(',').collect();
        task.insert("tags", tags_vec);
    } else {
        task.insert("tags", vec![""]);
    }

    let result = collection.insert_one(task).await;

    let response = if let Err(e) = result {
        AddTaskResponse {
            success: false,
            message: format!("Error while adding task: {}", e),
            task_id,
        }
    } else {
        AddTaskResponse {
            success: true,
            message: String::from("Successfully added task"),
            task_id,
        }
    };
    Json(response)
}
// render main tracker
#[get("/")]
fn main_page() -> Template {
    Template::render("index", context! {})
}

// fn new_habit(habit: Habit<'_>) ->

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();

    let mongo_pw =
        env::var("MONGO_INITDB_ROOT_PASSWORD").expect("Set MONGO_INITDB_ROOT_PASSWORD env");

    let mongo_url = format!("mongodb://root:{}@127.0.0.1:27017", mongo_pw);

    if env::var("ROCKET_PORT").is_err()
        && let Ok(port) = env::var("PORT")
    {
        let _ = env::set_var("ROCKET_PORT", port);
    }

    let db = Client::with_uri_str(mongo_url)
        .await
        .expect("Error connecting to mongodb")
        .database("tasks");

    rocket::build()
        // .attach(Mongo::init())
        .manage(db)
        .mount("/", routes![main_page])
        .mount("/", FileServer::from("static"))
        .mount("/add", routes![add_task])
        .mount("/wave", routes![wave, hello])
        .attach(Template::fairing())
}
