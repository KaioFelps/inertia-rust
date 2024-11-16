use std::sync::Arc;
use std::{collections::HashMap, sync::OnceLock};
use actix_web::{get, web::Data, App, HttpRequest, HttpServer, Responder};
use inertia_rust::{Component, Inertia, InertiaConfig, InertiaErrMapper, InertiaProp, InertiaVersion, SsrClient};
use inertia_rust::actix::{render_with_props, InertiaMiddleware};
use serde_json::json;
use vite_rust::{utils::resolve_path, Vite, ViteConfig};

#[get("/")]
async fn home(req: HttpRequest) -> impl Responder {
    let mut props = HashMap::new();
    props.insert("auth".into(), InertiaProp::Always(json!({
        "user": "Inertia-Rust"
    })));
    props.insert("message".into(), InertiaProp::Data("This message is sent from the server!".to_string().into()));

    render_with_props::<Vite>(&req, Component("Index".into()), props)
        .await
        .map_inertia_err()
}

#[get("/contact")]
async fn contact(req: HttpRequest) -> impl Responder {
    let mut props = HashMap::new();
    props.insert("user".into(), InertiaProp::Always(json!({
        "name": "John Doe",
        "email": "johndoe@example.com"
    })));

    render_with_props::<Vite>(&req, Component("Contact".into()), props)
        .await
        .map_inertia_err()
}

static VITE: OnceLock<Vite> = OnceLock::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init(); 

    let manifest_path = resolve_path(file!(), "../public/bundle/manifest.json");
    let vite_config = ViteConfig::new_with_defaults(&manifest_path);
    let vite = match Vite::new(vite_config).await {
        Ok(vite) => vite,
        Err(err) => panic!("{}", err)
    };

    let vite = VITE.get_or_init(move || vite);

    // Starts a Inertia manager instance with SSR enabled.
    let inertia: Inertia<Vite> = Inertia::new(
        InertiaConfig::builder()
            .set_url("http://localhost:8080")
            .set_version(InertiaVersion::Resolver(Box::new(|| vite.get_hash())))
            .set_template_path("www/root.html")
            .set_template_resolver(&inertia_rust::resolvers::basic_vite_resolver)
            .set_template_resolver_data(vite)
            .enable_ssr()
            .set_ssr_client(SsrClient::new("127.0.0.1", 1000))
            .build()
    )?;

    let inertia_data = Data::new(inertia);
    let inertia_data_to_move = Data::clone(&inertia_data);
    let server = HttpServer::new(move || {
        App::new()
        .app_data(inertia_data_to_move.clone())
        .wrap(InertiaMiddleware::new().with_shared_props(Arc::new(move |_req| {
            let mut shared_props = HashMap::new();
            shared_props.insert("version".into(), InertiaProp::Always("0.1.0".into()));
            shared_props.insert("assetsVersion".into(), InertiaProp::Lazy(Arc::new(|| {
                serde_json::to_value(vite.get_hash().to_string()).unwrap()
            })));

            shared_props
        })))
        .service(home)
        .service(contact)
        // serves vite assets from /assets path
        .service(actix_files::Files::new("/assets", "./public/bundle/assets").prefer_utf8(true))
        // serves public assets directly from / path
        // needs to be the last service because it's a wildcard
        .service(actix_files::Files::new("/", "./public/").prefer_utf8(true))
    }).bind(("127.0.0.1", 8080))?;
    
    // Starts a Node.js child process that runs the Inertia's server-side-rendering server.
    // It must be started after the server initialization to ensure that the server won't panic and
    // shutdown without killing Node process.
    let node = inertia_data.start_node_server("dist/ssr/ssr.js".into())?;
    
    let server = server.run().await;
    let _ = node.kill();
    println!("Inertia SSR server shutdown.");

    server
}
