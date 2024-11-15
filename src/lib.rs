mod http_method;
mod inertia;
mod utils;
mod page;
mod error;
mod req_type;
mod props;
mod providers;
mod features;
mod config;

pub mod node_process;

pub use inertia::Inertia as Inertia;
pub use inertia::Component;
pub use inertia::InertiaVersion;
pub use error::InertiaError;
pub use page::InertiaPage;
pub use page::InertiaSSRPage;
pub use inertia::ViewData;
pub use props::InertiaProps;
pub use props::InertiaProp;
pub use inertia::TemplateResolverOutput;
pub use inertia::SsrClient;
pub use config::{InertiaConfig, InertiaConfigBuilder};

pub use inertia::InertiaErrMapper;

#[cfg(feature = "actix")]
pub mod actix {
    pub use super::providers::actix::InertiaHeader;
    pub use super::providers::actix::facade::{render, render_with_props};
}

#[cfg(feature = "basic-vite-resolver")]
pub mod resolvers {
    pub use super::features::template_resolvers::basic_vite_resolver;
}
