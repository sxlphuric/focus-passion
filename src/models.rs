#[derive(FromForm, rocket::serde::Deserialize, rocket::serde::Serialize)]
pub struct Task {
    pub user_id: String,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub due: Option<u32>,
    pub project: String,
    pub section: Option<String>,
    pub tags: Vec<String>,
    pub completed: bool,
}

#[derive(FromForm, rocket::serde::Serialize, rocket::serde::Deserialize)]
pub struct ModifyTaskState {
    pub data: std::collections::HashMap<String, String>,
}
