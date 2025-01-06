# Server-Side Rendering

Inertia Rust supports server-side rendering. Enabling SSR gives your website better SEO, since the
very first page will be server-side rendered. Thereafter, they'll be ordinary SPA pages.

> Note: Node.js must be available in order to run the Inertia SSR server.

Enabling SSR is a very simple task. However, you must make few changes in your code so that the Node.js
process is correctly started and killed --- otherwise, the Node.js process would be left running and not
letting one start the new server on the given port.

First of all, enable SSR in your Inertia initialization function:

```rust
// src/config/inertia.rs
use super::vite::initialize_vite;
use inertia_rust::{
    template_resolvers::ViteTemplateResolver, Inertia, InertiaConfig, InertiaVersion, SsrClient,
};
use std::{env, io, sync::Arc};

pub async fn initialize_inertia() -> Result<Inertia, io::Error> {
    let vite = Arc::new(initialize_vite().await);
    let resolver = ViteTemplateResolver::new(vite.clone());

    Inertia::new(
        InertiaConfig::builder()
            .set_url("http://localhost:3000")
            .set_version(InertiaVersion::Literal(vite.get_hash().unwrap_or("development")))
            .set_template_path("www/root.html")
            .set_template_resolver(Box::new(resolver))
            
            // note these two lines

            .enable_ssr()
            // `set_ssr_client` is optional. If not set, `SsrClient::default()` will be used,
            // which is is "127.0.0.1:13714"
            .set_ssr_client(SsrClient::new("127.0.0.1", 1000))

            .build())
}
```

```rust
// src/main.rs
use std::sync::{Arc, OnceLock};
use actix_web::{dev::Path, web::Data, App, HttpServer};
use config::inertia::initialize_inertia;

mod config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    // starts a Inertia manager instance.
    let inertia = initialize_inertia().await?;
    let inertia = Data::new(inertia);
    let inertia_clone = inertia.clone();

    let server = HttpServer::new(move || App::new().app_data(inertia_clone.clone()))
        .bind(("127.0.0.1", 3000))?;

    // Starts a Node.js child process that runs the Inertia's server-side rendering server.
    // It must be started after the server initialization to ensure that the http server won't
    // panic and shutdown without killing the Node.Js process.
    let node = inertia.start_node_server("path/to/your/ssr.js".into())?;

    let server = server.run().await;
    let _ = node.kill().await;

    return server;
```

Indeed, you can replace `let _ = node.kill().await;` with `std::mem::drop(node.kill())`, but `.await`ing it
guarantees the process is killed.
