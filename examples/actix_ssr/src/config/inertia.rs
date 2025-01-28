use crate::ASSETS_VERSION;
use inertia_rust::{
    template_resolvers::ViteTemplateResolver, Inertia, InertiaConfig, InertiaError, InertiaVersion,
    SsrClient,
};
use std::io;

use super::vite::initialize_vite;

pub async fn initialize_inertia() -> Result<Inertia, io::Error> {
    let vite = initialize_vite().await;
    let resolver =
        ViteTemplateResolver::new(vite, "www/root.html").map_err(InertiaError::to_io_error)?;

    Inertia::new(
        InertiaConfig::builder()
            .set_url("http://localhost:8080")
            .set_version(InertiaVersion::Literal(ASSETS_VERSION.get().unwrap()))
            .set_template_resolver(Box::new(resolver))
            .enable_ssr()
            .set_ssr_client(SsrClient::new("127.0.0.1", 1000))
            .build(),
    )
}
