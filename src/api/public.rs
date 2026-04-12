use mongodb::bson;
use rocket::{State, http::CookieJar, serde::json::Json};

#[get("/get")]
pub async fn get_tasks(
    cookies: &CookieJar<'_>,
    db: &State<mongodb::Database>,
) -> Json<Vec<bson::Document>> {
    let user_id = cookies
        .get("uuid")
        .map(|crumb| crumb.value().to_string())
        .unwrap_or("error".to_string());

    let tasks = crate::db::fetch_tasks(db, &user_id, bson::Document::new()).await;

    Json(tasks)
}
