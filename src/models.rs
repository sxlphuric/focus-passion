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
    pub priority: Option<TaskPriority>,
}

#[derive(rocket::serde::Deserialize, rocket::serde::Serialize, FromFormField, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    #[field(value = "low")]
    Low,
    #[field(value = "medium")]
    Medium,
    #[field(value = "high")]
    High,
}

#[derive(FromForm, rocket::serde::Serialize, rocket::serde::Deserialize)]
pub struct ModifyTaskState {
    pub data: std::collections::HashMap<String, String>,
}
