#[derive(FromForm, rocket::serde::Deserialize, rocket::serde::Serialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub due: Option<u32>,
    pub project: String,
    pub section: Option<String>,
    pub tags: Vec<String>,
    pub completed: bool,
}
