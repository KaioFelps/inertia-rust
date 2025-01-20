use serde_json::{Map, Value};

pub mod encrypt_middleware;
pub mod facade;
pub mod headers;
pub mod impls;
pub mod middleware;

struct CustomViewData(Map<String, Value>);
