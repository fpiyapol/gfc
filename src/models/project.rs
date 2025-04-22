use serde::Serialize;

#[derive(Serialize)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub status: String,
}
