use nanoid::nanoid;
use rocket::{State, form::Form, http::CookieJar, serde::json::Json};

use crate::{AddTaskResponse, TaskOptions};

#[post("/add", data = "<opt>")]
#[allow(unused_mut)]
pub async fn add_task(
    cookies: &CookieJar<'_>,
    opt: Form<TaskOptions<'_>>,
    db: &State<mongodb::Database>,
) -> Json<AddTaskResponse> {
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
        id: task_id.clone(),
        name: opt.name.to_string(),
        description: opt.description.map(|s| s.to_string()),
        due: opt.due,
        section: opt.section.map(|s| s.to_string()),
        project: opt.project.to_string(),
        tags: tags_vec,
        completed: opt.completed.unwrap_or(false),
    };

    let result = crate::db::insert_task(db, &user_id, &task);

    let (success, message) = match result.await {
        Ok(_) => (true, String::from("Successfully added task")),
        Err(e) => (false, format!("Error: {}", e)),
    };

    Json(AddTaskResponse {
        success,
        message,
        task_id: task_id.clone(),
    })
}
