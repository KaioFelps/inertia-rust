use serde::Serialize;
use serde_json::{Map, Value};

/// `InertiaTemporarySession` struct contains data that InertiaMiddleware will try to extract
/// from the request and merge with shared props.
///
/// You must inject it by yourself by a second middleware, which gets these information from
/// your framework sessions manager.
#[derive(Clone, Serialize)]
pub struct InertiaTemporarySession {
    pub errors: Option<Map<String, Value>>,
    pub prev_req_url: String,
}
