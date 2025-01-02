use crate::{routes::register_routes, ASSETS_VERSION};
use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    App,
};
use inertia_rust::{actix::InertiaMiddleware, hashmap, prop_resolver, InertiaProp};
use serde_json::to_value;
use std::{path::Path, sync::Arc};

pub fn get_server() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let mut app = App::new()
        .wrap(
            InertiaMiddleware::new().with_shared_props(Arc::new(move |_req| {
                hashmap![
                    "version" => InertiaProp::always("0.1.0").unwrap(),
                    "assetsVersion" => InertiaProp::lazy(prop_resolver!({to_value(ASSETS_VERSION.get().unwrap()).unwrap()}))
                ]
            })),
        )
        .configure(register_routes);

    // adds this service only if the directory exists
    if Path::new("public/bundle/assets").exists() {
        // serves vite assets from /assets path
        app = app.service(
            actix_files::Files::new("/assets", "./public/bundle/assets").prefer_utf8(true),
        );
    }

    // serves public assets directly from / path
    // needs to be the last service because it's a wildcard
    app.service(actix_files::Files::new("/", "./public/").prefer_utf8(true))
}
