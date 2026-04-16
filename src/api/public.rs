use crate::{AddTaskResponse, TaskOptions, db};
use mongodb::bson;
use nanoid::nanoid;
use rocket::{Route, State, form::Form, http::CookieJar, serde::json::Json};
use rocket_dyn_templates::{Template, context};

pub fn routes() -> Vec<Route> {
    routes![
        get_tasks,
        add_task,
        remove_task,
        complete_task,
        modify_task,
        fetch_tasks_complete_filtering,
    ]
}

#[get("/get")]
pub async fn get_tasks(
    cookies: &CookieJar<'_>,
    db: &State<mongodb::Database>,
) -> Json<Vec<crate::models::Task>> {
    let user_id = cookies
        .get("uuid")
        .map(|crumb| crumb.value().to_string())
        .unwrap_or("error".to_string());

    let tasks = crate::db::fetch_tasks(db, bson::doc! { "user_id": user_id }).await;

    Json(tasks)
}

#[post("/add", data = "<opt>")]
#[allow(unused_mut)]
pub async fn add_task(
    cookies: &CookieJar<'_>,
    opt: Form<TaskOptions<'_>>,
    db: &State<mongodb::Database>,
) -> Template {
    let user_id = cookies
        .get("uuid")
        .map(|crumb| crumb.value().to_string())
        .unwrap_or("error".to_string());
    let task_id = nanoid!();

    let tags_vec: Vec<String> = opt
        .tags
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let task = crate::models::Task {
        user_id: user_id.clone(),
        id: task_id.clone(),
        name: opt.name.to_string(),
        description: opt.description.map(|s| s.to_string()),
        due: opt.due,
        section: opt.section.map(|s| s.to_string()),
        project: opt.project.to_string(),
        tags: tags_vec,
        completed: opt.completed.unwrap_or(false),
        priority: opt.priority,
    };

    let result = crate::db::insert_task(db, &task);

    let (success, message) = match result.await {
        Ok(_) => (true, String::from("Successfully added task")),
        Err(e) => (false, format!("Error: {}", e)),
    };

    let _response = Json(AddTaskResponse {
        success,
        message,
        task_id: task_id.clone(),
    });

    let projects = crate::db::get_unique_projects(db, &user_id)
        .await
        .unwrap_or_default();

    Template::render("fragments/add_task_response", context! { task, projects })
}

#[post("/remove/<id>")]
pub async fn remove_task(
    id: &str,
    db: &State<mongodb::Database>,
    cookies: &CookieJar<'_>,
) -> &'static str {
    let user_id = cookies.get("uuid").map(|c| c.value()).unwrap_or("error");

    let _ = db::delete_task(db, user_id, id).await;

    ""
}

#[post("/complete/<id>")]
pub async fn complete_task(
    id: &str,
    db: &State<mongodb::Database>,
    cookies: &CookieJar<'_>,
) -> &'static str {
    let user_id = cookies.get("uuid").map(|c| c.value()).unwrap_or("error");

    let _task = db::toggle_completed_state(db, user_id, id).await;

    // Template::render("task_checkbox", context! { task: task.unwrap() })
    ""
}

#[post("/modify/<id>/<param>", data = "<state>")]
pub async fn modify_task(
    id: &str,
    db: &State<mongodb::Database>,
    cookies: &CookieJar<'_>,
    param: &str,
    state: Form<crate::models::ModifyTaskState>,
) -> Template {
    let user_id = cookies.get("uuid").map(|c| c.value()).unwrap_or("error");

    let val = state.data.get(param).cloned().unwrap_or_default();

    let val_bson = if val == "none" {
        bson::Bson::Null
    } else {
        bson::Bson::String(val)
    };

    let task = db::modify_task(db, user_id, id, param, val_bson)
        .await
        .unwrap()
        .unwrap();

    Template::render("task_item", context! {task})
}

#[get("/list?<status>")]
pub async fn fetch_tasks_complete_filtering(
    status: Option<&str>,
    cookies: &CookieJar<'_>,
    db: &State<mongodb::Database>,
) -> Template {
    let user_id = cookies.get("uuid").map(|c| c.value()).unwrap_or("error");

    let predicate = match status {
        Some("completed") => bson::doc! { "user_id" : user_id, "completed": true},
        _ => bson::doc! { "user_id" : user_id, "completed": false },
    };

    let tasks = crate::db::fetch_tasks(db, predicate).await;

    Template::render("fragments/tasks_view", context! {tasks})
}
