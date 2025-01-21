use std::sync::OnceLock;

use actix_web::{web::Data, HttpServer};
use config::inertia::initialize_inertia;
use server::get_server;

mod config;
mod domain;
mod dtos;
mod routes;
mod server;

static ASSETS_VERSION: OnceLock<&str> = OnceLock::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    // Starts a Inertia manager instance with SSR enabled.
    let inertia = initialize_inertia().await?;
    let inertia = Data::new(inertia);
    let inertia_clone = inertia.clone();

    let server = HttpServer::new(move || get_server().app_data(inertia_clone.clone()))
        .bind(("127.0.0.1", 8080))?;

    // Starts a Node.js child process that runs the Inertia's server-side-rendering server.
    // It must be started after the server initialization to ensure that the server won't panic and
    // shutdown without killing Node process.
    let node = inertia.start_node_server("dist/ssr/ssr.js".into())?;

    let server = server.run().await;

    // in production, await node.kill() instead of dropping it,
    // so that you assure server has shutdown
    std::mem::drop(node.kill());

    println!("Inertia SSR server shutdown.");

    server
}
