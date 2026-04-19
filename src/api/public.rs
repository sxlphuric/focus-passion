use crate::{
    TaskOptions, db,
    models::{NaiveDateForm, TaskPriority},
};
use mongodb::bson;
use nanoid::nanoid;
use rocket::{
    Route, State,
    form::Form,
    http::{CookieJar, Status},
    serde::json::Json,
};
use rocket_dyn_templates::{Template, context};
use serde_with::chrono::NaiveDate;

pub fn routes() -> Vec<Route> {
    routes![
        get_tasks,
        add_task,
        remove_task,
        complete_task,
        modify_task,
        fetch_tasks_complete_filtering,
        search_tasks
    ]
}

#[get("/get")]
pub async fn get_tasks(
    cookies: &CookieJar<'_>,
    db: &State<mongodb::Database>,
) -> Result<Json<Vec<crate::models::Task>>, Status> {
    let user_id = match cookies.get("uuid") {
        Some(crumb) => crumb.value(),
        None => return Err(Status::Unauthorized),
    };

    let tasks = crate::db::fetch_tasks(db, bson::doc! { "user_id": user_id }).await;

    Ok(Json(tasks))
}

#[derive(FromForm, rocket::serde::Serialize, rocket::serde::Deserialize)]
pub struct TaskSearchOptions<'r> {
    due: &'r str,
    priority: Option<TaskPriority>,
}

#[get("/search?<project>&<opt..>")]
pub async fn search_tasks(
    cookies: &CookieJar<'_>,
    db: &State<mongodb::Database>,
    project: &str,
    opt: TaskSearchOptions<'_>,
) -> Result<Json<Vec<crate::models::Task>>, Status> {
    let user_id = match cookies.get("uuid") {
        Some(crumb) => crumb.value(),
        None => return Err(Status::Unauthorized),
    };

    let mut task_filter = bson::Document::new();

    task_filter.insert("user_id", user_id);

    if !opt.due.is_empty() {
        task_filter.insert("due", opt.due);
    }

    if !opt.project.is_empty() {
        task_filter.insert("project", opt.project);
    }

    if let Some(prio) = opt.priority
        && let Ok(bson_prio) = bson::to_bson(&prio)
    {
        task_filter.insert("priority", bson_prio);
    }

    let tasks = crate::db::fetch_tasks(db, task_filter).await;

    Ok(Json(tasks))
}

#[post("/add", data = "<opt>")]
#[allow(unused_mut)]
pub async fn add_task(
    cookies: &CookieJar<'_>,
    opt: Form<TaskOptions<'_>>,
    db: &State<mongodb::Database>,
) -> Result<Template, Status> {
    let user_id = match cookies.get("uuid") {
        Some(crumb) => crumb.value(),
        None => return Err(Status::Unauthorized),
    };
    let task_id = nanoid!();

    let tags_vec: Vec<String> = opt
        .tags
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let due_date_parsed = opt.due.filter(|s| !s.is_empty()).map(|due| {
        NaiveDate::parse_from_str(&due, "%Y-%m-%d")
            .map(NaiveDateForm)
            .expect("Invalid date format ...")
    });

    let task = crate::models::Task {
        user_id: user_id.to_string(),
        id: task_id.clone(),
        name: opt.name.to_string(),
        description: opt.description.map(|s| s.to_string()),
        due: due_date_parsed,
        section: opt.section.map(|s| s.to_string()),
        project: opt.project.to_string(),
        tags: tags_vec,
        completed: opt.completed.unwrap_or(false),
        priority: opt.priority,
    };

    let result = crate::db::insert_task(db, &task).await;

    match result {
        Ok(_) => {
            let projects = crate::db::get_unique_projects(db, user_id)
                .await
                .unwrap_or_default();
            Ok(Template::render(
                "fragments/add_task_response",
                context! { task, projects },
            ))
        }
        Err(e) => {
            eprintln!("Db error: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[post("/remove/<id>")]
pub async fn remove_task(
    id: &str,
    db: &State<mongodb::Database>,
    cookies: &CookieJar<'_>,
) -> Result<&'static str, Status> {
    let user_id = match cookies.get("uuid") {
        Some(crumb) => crumb.value(),
        None => return Err(Status::Unauthorized),
    };

    let result = db::delete_task(db, user_id, id).await;

    match result {
        Ok(_) => Ok(""),
        Err(e) => {
            eprintln!("Db error: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[post("/complete/<id>")]
pub async fn complete_task(
    id: &str,
    db: &State<mongodb::Database>,
    cookies: &CookieJar<'_>,
) -> Result<&'static str, Status> {
    let user_id = match cookies.get("uuid") {
        Some(crumb) => crumb.value(),
        None => return Err(Status::Unauthorized),
    };
    let result = db::toggle_completed_state(db, user_id, id).await;

    // Template::render("task_checkbox", context! { task: task.unwrap() })

    match result {
        Ok(_) => Ok(""),
        Err(e) => {
            eprintln!("Db error: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[post("/modify/<id>/<param>", data = "<state>")]
pub async fn modify_task(
    id: &str,
    db: &State<mongodb::Database>,
    cookies: &CookieJar<'_>,
    param: &str,
    state: Form<crate::models::ModifyTaskState>,
) -> Result<Template, Status> {
    let user_id = match cookies.get("uuid") {
        Some(crumb) => crumb.value(),
        None => return Err(Status::Unauthorized),
    };

    let val = state.data.get(param).cloned().unwrap_or_default();

    let result = db::modify_task(db, user_id, id, param, val).await;

    match result {
        Ok(task) => Ok(Template::render("task_item", context! {task})),
        Err(e) => {
            eprintln!("Db error: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[get("/list?<status>")]
pub async fn fetch_tasks_complete_filtering(
    status: Option<&str>,
    cookies: &CookieJar<'_>,
    db: &State<mongodb::Database>,
) -> Result<Template, Status> {
    let user_id = match cookies.get("uuid") {
        Some(crumb) => crumb.value(),
        None => return Err(Status::Unauthorized),
    };

    let predicate = match status {
        Some("completed") => bson::doc! { "user_id" : user_id, "completed": true},
        _ => bson::doc! { "user_id" : user_id, "completed": false },
    };

    let tasks = crate::db::fetch_tasks(db, predicate).await;

    Ok(Template::render("fragments/tasks_view", context! {tasks}))
}
