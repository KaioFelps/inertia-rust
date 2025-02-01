use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Deserialize, Serialize, Clone, Debug)]
pub struct CreateTask {
    #[validate(
        required(message = "Title content is a mandatory field."),
        length(min = 5, message = "Task title must be at least 5 characters long.")
    )]
    pub title: Option<String>,

    #[validate(
        required(message = "Task content is a mandatory field."),
        length(
            min = 10,
            message = "Task content must be at least 10 characters long."
        )
    )]
    pub content: Option<String>,
}
