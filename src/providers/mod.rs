#[cfg(feature = "actix")]
mod actix_provider;

#[cfg(feature = "actix")]
pub mod actix {
    pub use super::actix_provider::{
        encrypt_middleware, facade::is_inertia_response, headers, impls::SessionErrors, middleware,
    };
}
