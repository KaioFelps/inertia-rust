use crate::dtos::task::CreateTask;
use actix_web::{
    get, post,
    web::{self, Json, Redirect},
    HttpRequest, Responder,
};
use inertia_rust::{
    hashmap, prop_resolver, validators::InertiaValidateOrRedirect, Inertia, InertiaFacade,
    InertiaProp, InertiaService, IntoInertiaPropResult,
};
use serde::Deserialize;
use serde_json::json;

use crate::domain::tasks::service::get_tasks;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(home)
        .service(contact)
        .inertia_route("/foo", "Foo/Index")
        .service(r#todo)
        .inertia_route("/todo/create", "Todo/Create")
        .service(store_task);
}

#[post("/todo/store")]
async fn store_task(req: HttpRequest, body: Json<CreateTask>) -> impl Responder {
    let payload = match body.validate_or_back(&req) {
        Err(err_redirect) => {
            println!("errors");
            return err_redirect;
        }
        Ok(payload) => payload,
    };

    println!("Task successfully created: {:#?}", payload);

    Redirect::to("/todo").see_other()
}

#[get("/")]
async fn home(req: HttpRequest) -> impl Responder {
    let props = hashmap![
        "auth" => InertiaProp::always(json!({ "user": "Inertia-Rust" })),
        "message" => InertiaProp::data("This message is sent from the server!"),
    ];

    Inertia::render_with_props(&req, "Index".into(), props).await
}

#[get("/contact")]
async fn contact(req: HttpRequest) -> impl Responder {
    let props = hashmap![
        "user" => InertiaProp::always(json!({
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
                tasks.into_inertia_value()
            })).into_mergeable(),
            "page" => InertiaProp::data(page)
        ],
    )
    .await
}
