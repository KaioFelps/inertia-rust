use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Deserialize, Serialize, Clone, Debug)]
pub struct CreateTask {
    #[validate(
        required(message = "Task content is a mandatory field."),
        length(
            min = 10,
            message = "Task content must be at least 10 characters long."
        )
    )]
    content: Option<String>,
}
