#[derive(Debug)]
pub enum InertiaError {
    SerializationError(String),
    HeaderError(String),
    SsrError(String),
}