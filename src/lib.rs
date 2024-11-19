mod config;
mod error;
mod features;
mod http_method;
mod inertia;
mod page;
mod props;
mod providers;
mod req_type;
mod temporary_session;
mod utils;

pub mod node_process;

pub use config::{InertiaConfig, InertiaConfigBuilder};
pub use error::InertiaError;
pub use inertia::Component;
pub use inertia::Inertia;
pub use inertia::InertiaService;
pub use inertia::InertiaVersion;
pub use inertia::SsrClient;
pub use inertia::TemplateResolverOutput;
pub use inertia::ViewData;
pub use page::InertiaPage;
pub use page::InertiaSSRPage;
pub use props::InertiaProp;
pub use props::InertiaProps;
pub use temporary_session::InertiaTemporarySession;

#[cfg(feature = "actix")]
pub mod actix {
    pub use super::providers::actix::facade::{render, render_with_props};
    pub use super::providers::actix::headers::InertiaHeader;
    pub use super::providers::actix::middleware::InertiaMiddleware;
}

#[cfg(feature = "basic-vite-resolver")]
pub mod resolvers {
    pub use super::features::template_resolvers::basic_vite_resolver;
}
