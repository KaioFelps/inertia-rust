#[cfg(feature = "actix")]
mod actix_provider;

#[cfg(feature = "actix")]
pub mod actix {
    pub use super::actix_provider::{facade, headers, middleware};
}
