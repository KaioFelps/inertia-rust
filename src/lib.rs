mod config;
mod error;
mod facade;
mod features;
mod http_method;
mod inertia;
mod macros;
mod page;
mod props;
mod providers;
mod req_type;
mod template_resolver;
mod temporary_session;
mod utils;

pub mod node_process;

pub use config::{InertiaConfig, InertiaConfigBuilder};
pub use error::{InertiaError, IntoInertiaError};
pub use facade::InertiaFacade;
pub use inertia::{Component, Inertia, InertiaService, InertiaVersion, SsrClient, ViewData};
pub use page::{InertiaPage, InertiaSSRPage};
pub use props::{InertiaProp, InertiaProps, IntoInertiaPropResult};
pub use temporary_session::{InertiaSessionToReflash, InertiaTemporarySession};

#[cfg(feature = "actix")]
pub mod actix {
    pub use super::providers::actix::encrypt_middleware::EncryptHistoryMiddleware;
    pub use super::providers::actix::headers::InertiaHeader;
    pub use super::providers::actix::is_inertia_response;
    pub use super::providers::actix::middleware::InertiaMiddleware;
    pub use super::providers::actix::SessionErrors;
}

#[cfg(feature = "vite-template-resolver")]
pub mod template_resolvers {
    pub use super::features::template_resolvers::ViteTemplateResolver;
    pub use super::template_resolver::TemplateResolver;
}

#[cfg(feature = "validator")]
pub mod validators {
    pub use super::features::validators::validator::InertiaValidateOrRedirect;

    #[cfg(feature = "actix-validator")]
    pub use super::features::validators::validator::actix_validator;
}
