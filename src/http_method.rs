use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) enum HttpMethod {
    #[serde(rename="get")]
    Get,
    #[serde(rename="post")]
    Post,
    #[serde(rename="put")]
    Put,
    #[serde(rename="patch")]
    Patch,
    #[serde(rename="delete")]
    Delete
}
