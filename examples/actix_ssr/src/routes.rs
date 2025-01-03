use actix_web::{get, web, HttpRequest, Responder};
use inertia_rust::{hashmap, prop_resolver, Inertia, InertiaFacade, InertiaProp, InertiaService};
use serde::Deserialize;
use serde_json::{json, to_value};

use crate::domain::tasks::service::get_tasks;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(home)
        .service(contact)
        .inertia_route("/foo", "Foo/Index")
        .service(r#todo);
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

#[derive(Deserialize)]
struct TodoQuery {
    page: Option<usize>,
}

#[get("/todo")]
async fn r#todo(req: HttpRequest, query: web::Query<TodoQuery>) -> impl Responder {
    let page = query.page.unwrap_or(1);

    Inertia::render_with_props(
        &req,
        "Todo/Index".into(),
        hashmap![
            "tasks" => InertiaProp::defer(prop_resolver!({
                let tasks = get_tasks(page).await;
                to_value(tasks).unwrap()
            })).into_mergeable(),
            "page" => InertiaProp::data(page)
        ],
    )
    .await
}
