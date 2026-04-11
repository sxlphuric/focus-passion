use mongodb::{Client, bson};
use nanoid::nanoid;
use rocket::{
    State, form::Form, fs::FileServer, futures::TryStreamExt, http::CookieJar, response::Redirect,
    serde::json::Json,
};
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

#[derive(FromForm, rocket::serde::Deserialize)]
struct TaskOptions<'r> {
    user_id: &'r str,
    name: &'r str,
    description: Option<&'r str>,
    due: Option<u32>,
    project: Option<&'r str>,
    section: Option<&'r str>,
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

#[post("/add", data = "<opt>")]
#[allow(unused_mut)]
async fn add_task(opt: Form<TaskOptions<'_>>, db: &State<mongodb::Database>) -> Template {
    let collection = db.collection::<bson::Document>(opt.user_id);

    let task_id: String = nanoid!();

    let mut task = bson::Document::new();

    task.insert("id", &task_id);
    task.insert("name", opt.name);
    task.insert("description", opt.description);
    task.insert("due", opt.due);
    task.insert("section", opt.section);
    task.insert("project", opt.project);
    task.insert("completed", false);

    if let Some(tags) = opt.tags {
        let tags_string = String::from(tags);
        let tags_vec: Vec<String> = tags_string
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        task.insert("tags", tags_vec);
    } else {
        task.insert("tags", Vec::<String>::new());
    }

    let result = collection.insert_one(&task).await;

    let (success, message) = match result {
        Ok(_) => (true, String::from("Successfully added task")),
        Err(e) => (false, format!("Error: {}", e)),
    };

    let _response = Json(AddTaskResponse {
        success,
        message,
        task_id,
    });

    Template::render("task_item", context! { task })
}

async fn fetch_user_tasks(db: &State<mongodb::Database>, user_id: &str) -> Vec<bson::Document> {
    let collection = db.collection::<bson::Document>(user_id);

    let mut cursor = collection.find(bson::Document::new()).await.unwrap();

    let mut documents: Vec<bson::Document> = Vec::new();

    while let Some(doc) = cursor.try_next().await.unwrap() {
        documents.push(doc);
    }

    documents
}

#[get("/get")]
async fn get_tasks(
    cookies: &CookieJar<'_>,
    db: &State<mongodb::Database>,
) -> Json<Vec<bson::Document>> {
    let user_id = cookies
        .get("uuid")
        .map(|crumb| crumb.value().to_string())
        .unwrap_or("error".to_string());

    let tasks = fetch_user_tasks(db, &user_id).await;

    Json(tasks)
}

// render main tracker
#[get("/")]
async fn main_page(cookies: &CookieJar<'_>, db: &State<mongodb::Database>) -> Template {
    let user_id = cookies
        .get("uuid")
        .map(|crumb| crumb.value().to_string())
        .unwrap_or("error".to_string());

    let tasks = fetch_user_tasks(db, &user_id).await;

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
        .mount("/", routes![main_page, add_task, get_tasks])
        .mount("/", FileServer::from("static"))
        .mount("/wave", routes![wave, hello])
        .attach(Template::fairing())
}
