use std::{error::Error, fmt, io};

use crate::node_process::NodeJsError;

#[derive(Debug)]
pub enum InertiaError {
    SerializationError(String),
    HeaderError(String),
    SsrError(String),
    RenderError(String),
    NodeJsError(NodeJsError),
}

impl fmt::Display for InertiaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Inertia Error: {}", self.get_cause())
    }
}

impl Error for InertiaError {}

impl InertiaError {
    pub fn get_cause(&self) -> String {
        match self {
            InertiaError::HeaderError(err) => err.clone(),
            InertiaError::NodeJsError(node_err) => {
                format!("{} ({})", node_err.get_cause(), node_err.get_description())
            }
            InertiaError::SerializationError(err) => err.clone(),
            InertiaError::SsrError(err) => err.clone(),
            InertiaError::RenderError(err) => err.clone(),
        }
    }

    pub fn to_io_error(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, self.get_cause())
    }
}
