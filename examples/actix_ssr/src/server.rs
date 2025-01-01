use crate::{routes::register_routes, ASSETS_VERSION};
use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    App,
};
use inertia_rust::{actix::InertiaMiddleware, hashmap, InertiaProp, IntoPropResolver};
use serde_json::to_value;
use std::sync::Arc;

pub fn get_server() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .wrap(
            InertiaMiddleware::new().with_shared_props(Arc::new(move |_req| {
                hashmap![
                    "version" => InertiaProp::always("0.1.0").unwrap(),
                    "assetsVersion" => InertiaProp::lazy((|| to_value(ASSETS_VERSION.get().unwrap()).unwrap()).wrap_with_arc())
                ]
            })),
        )
        .configure(register_routes)
        // serves vite assets from /assets path
        .service(actix_files::Files::new("/assets", "./public/bundle/assets").prefer_utf8(true))
        // serves public assets directly from / path
        // needs to be the last service because it's a wildcard
        .service(actix_files::Files::new("/", "./public/").prefer_utf8(true))
}
