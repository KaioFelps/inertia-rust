use std::{collections::HashMap, sync::OnceLock};
use actix_web::{get, web::Data, App, HttpRequest, HttpResponse, HttpServer, Responder};
use inertia_rust::{Component, Inertia, InertiaProp, InertiaVersion, SsrClient};
use serde_json::json;
use vite_rust::{utils::resolve_path, Vite, ViteConfig};

#[get("/")]
async fn home(req: HttpRequest) -> impl Responder {
    let mut props = HashMap::new();
    props.insert("version".into(), InertiaProp::Always("0.1.0".into()));
    props.insert("auth".into(), InertiaProp::Always(json!({
        "user": "Inertia-Rust"
    })));
    props.insert("message".into(), InertiaProp::Data("This message is sent from the server!".to_string().into()));

    inertia_rust::render_with_props::<Vite>(&req, Component("Index".into()), props).await
    .unwrap_or(HttpResponse::InternalServerError().finish())
}

#[get("/contact")]
async fn contact(req: HttpRequest) -> impl Responder {
    let mut props = HashMap::new();
    props.insert("user".into(), InertiaProp::Always(json!({
        "name": "John Doe",
        "email": "johndoe@example.com"
    })));

    inertia_rust::render_with_props::<Vite>(&req, Component("Contact".into()), props).await
    .unwrap_or(HttpResponse::InternalServerError().finish())
}

static VITE: OnceLock<Vite> = OnceLock::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let manifest_path = resolve_path(file!(), "../public/bundle/manifest.json");
    let vite_config = ViteConfig::new_with_defaults(&manifest_path);
    let vite = match Vite::new(vite_config).await {
        Ok(vite) => vite,
        Err(err) => panic!("{}", err)
    };

    let vite = VITE.get_or_init(move || vite);

    // Starts a Inertia manager instance with SSR enabled.
    let inertia: Inertia<Vite> = Inertia::new_with_ssr(
        "http://localhost:8080",
        InertiaVersion::Literal(vite.get_hash().to_string()),
        "www/root.html",
        &inertia_rust::template_resolvers::basic_vite_resolver,
        vite,
        Some(SsrClient::new("127.0.0.1", 1000))
    ).await?;

    let inertia_data = Data::new(inertia);
    
    let inertia_data_to_move = Data::clone(&inertia_data);
    let server = HttpServer::new(move || {
        App::new()
        .app_data(Data::clone(&inertia_data_to_move))
        .service(home)
        .service(contact)
        // serves vite assets from /assets path
        .service(actix_files::Files::new("/assets", "./public/bundle/assets").prefer_utf8(true))
        // serves public assets directly from / path
        // needs to be the last service because it's a wildcard
        .service(actix_files::Files::new("/", "./public/").prefer_utf8(true))
    }).bind(("127.0.0.1", 8080))?;
    
    // Starts a Node.js child process that runs the Inertia's server-side-rendering server.
    // It must be after server creation to guarantee that server won't panic and stop node child process
    // from being killed
    let node = inertia_data.start_node_server("dist/ssr/ssr.js".into())?;
    
    let server = server.run().await;
    let _ = node.kill();
    println!("Inertia SSR server shutdown.");

    server
}
