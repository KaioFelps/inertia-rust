use actix_web::{get, web, HttpRequest, Responder};
use inertia_rust::{hashmap, Inertia, InertiaFacade, InertiaProp, InertiaService};
use serde_json::json;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(home)
        .service(contact)
        .inertia_route("/foo", "Foo/Index");
}

#[get("/")]
async fn home(req: HttpRequest) -> impl Responder {
    let props = hashmap![
        "auth" => InertiaProp::Always(json!({ "user": "Inertia-Rust" })),
        "message" => InertiaProp::Data("This message is sent from the server!".to_string().into()),
    ];

    Inertia::render_with_props(&req, "Index".into(), props).await
}

#[get("/contact")]
async fn contact(req: HttpRequest) -> impl Responder {
    let props = hashmap![
        "user" => InertiaProp::Always(json!({
            "name": "John Doe",
            "email": "johndoe@example.com"
        }))
    ];

    Inertia::render_with_props(&req, "Contact".into(), props).await
}
