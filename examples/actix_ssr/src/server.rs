use crate::{
    config::file_session::FileSessionStore,
    middlewares::{GarbageCollectorMiddleware, ReflashTemporarySessionMiddleware},
    routes::register_routes,
    ASSETS_VERSION,
};
use actix_session::{SessionExt, SessionMiddleware};
use actix_web::{
    cookie::{Key, SameSite},
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    App,
};
use inertia_rust::{
    actix::InertiaMiddleware, hashmap, prop_resolver, InertiaProp, IntoInertiaPropResult,
};
use serde_json::{Map, Value};
use std::{env, sync::Arc};

pub fn get_server() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let rust_env = env::var("RUST_ENV").unwrap_or("production".into());
    let use_secure_cookie = rust_env.as_str() == "production";

    let key = Key::from(
        env::var("APP_KEY")
            .expect("You must provide a valid APP_KEY environment variable.")
            .as_bytes(),
    );

    let storage = FileSessionStore::default();

    App::new()
        .wrap(GarbageCollectorMiddleware::new())
        .wrap(
            InertiaMiddleware::new().with_shared_props(Arc::new(move |req| {
                let flash = req.get_session()
                    .get::<Map<String, Value>>("_flash")
                    .unwrap_or_default()
                    .unwrap_or_default();
                
                Box::pin(async move {
                    hashmap![
                        "version" => InertiaProp::always("0.1.0"),
                        "assetsVersion" => InertiaProp::lazy(prop_resolver!({ ASSETS_VERSION.get().unwrap().into_inertia_value() })),
                        "flash" => InertiaProp::always(flash)
                    ]
                })})),
        )
        .wrap(ReflashTemporarySessionMiddleware::new())
        .wrap(SessionMiddleware::builder(storage.clone(), key.clone())
        .cookie_domain(Some("localhost".into()))
        .cookie_http_only(true)
        .cookie_same_site(SameSite::Strict)
        .cookie_name("test_cookie_session_id".into())
        .cookie_secure(use_secure_cookie)
        .build(),)
        .configure(register_routes)
        // serves public assets directly from / path
        // needs to be the last service because it's a wildcard
        .service(actix_files::Files::new("/", "./public/").prefer_utf8(true))
}
