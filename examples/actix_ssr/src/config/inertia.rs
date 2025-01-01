use std::io;

use inertia_rust::{
    resolvers::BasicViteResolver, Inertia, InertiaConfig, InertiaVersion, SsrClient,
};

use crate::ASSETS_VERSION;

use super::vite::initialize_vite;

pub async fn initialize_inertia() -> Result<Inertia, io::Error> {
    let vite = initialize_vite().await;
    let resolver = BasicViteResolver::new(vite);

    Inertia::new(
        InertiaConfig::builder()
            .set_url("http://localhost:8080")
            .set_version(InertiaVersion::Literal(ASSETS_VERSION.get().unwrap()))
            .set_template_path("www/root.html")
            .set_template_resolver(Box::new(resolver))
            .enable_ssr()
            .set_ssr_client(SsrClient::new("127.0.0.1", 1000))
            .build(),
    )
}
