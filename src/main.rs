use rocket_dyn_templates::{Template, context};
use mongodb::Client
use std::sync::Mutex;

#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

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
struct HabitOptions<'r> {
    description: Option<&'r str>,
    tag: Option<&'r str>,
}

struct Tasks<'r> {
    tasks: Mutex<Vec<HabitOptions<'r>>>,
}

let client = Client::with_uri_str("mongodb://127.0.0.1:27017").await?;

// Try visiting:
//   http://127.0.0.1:8000/hello/world
#[get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}

// Try visiting:
//   http://127.0.0.1:8000/hello/мир
#[get("/мир")]
fn mir() -> &'static str {
    "Привет, мир!"
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

#[get("/?<name>&<opt..>")]
fn add_task(name: String, opt: HabitOptions<'_>) {
    let client_ref = client.clone();

    tokio::task::spawn(async move {
        let collection = client_ref.database("items").collection::<Document>(&format!("coll{}", i));

        // Do something with the collection
    });
}
}
// render main tracker
#[get("/")]
fn main_page() -> Template {
    Template::render("index", context! { })
}

// fn new_habit(habit: Habit<'_>) ->

#[launch]
fn rocket() -> _ {
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "3000".into()).parse().unwrap();
    let config = rocket::Config {
        port,
        ..rocket::Config::default()
    };
    rocket::custom(&config)
        .mount("/", routes![main_page])
        .mount("/public", FileServer::from(relative!(static)))
        .mount("/add", routes![add_task])
        .mount("/hello", routes![world, mir])
        .mount("/wave", routes![wave])
        .attach(Template::fairing())
}
