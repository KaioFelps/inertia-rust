use crate::node_process::NodeJsError;

#[derive(Debug)]
pub enum InertiaError {
    SerializationError(String),
    HeaderError(String),
    SsrError(String),
    NodeJsError(NodeJsError)
}