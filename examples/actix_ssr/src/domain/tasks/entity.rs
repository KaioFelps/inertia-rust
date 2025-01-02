use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Task {
    pub title: String,
    pub description: String,
    pub done: bool,
}
