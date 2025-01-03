# Shared Props

There are some props that are sent to the front-end on every request. These can be easily shared through
`InertiaMiddleware` rather than conventional props on each request.

The middleware contains a `with_shared_props` method, which requires a callback --- wrapped in an `Arc`
--- that receives a reference to the current request. You can use it to extract any information you might
want to share.

```rust
use actix_web::{App, HttpServer};
use inertia_rust::actix::InertiaMiddleware;
use inertia_rust::{hashmap, InertiaProp, InertiaProps, InertiaService};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ...

    HttpServer::new(move || {
        App::new()
            .app_data(inertia.clone())
            .wrap(InertiaMiddleware::new().with_shared_props(Arc::new(|req| {
                // get the sessions from the request
                // depending on your opted framework
                let session = req.get_session();
                let flash = serde_json::to_value(session.get::<String>("flash").unwrap()).unwrap();

                hashmap![
                    "flash" => InertiaProp::data(flash)
                ]
            })))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
```

Although you cannot do asynchronous code inside this callback, you can still use them inside lazy
`InertiaProp` variants (`Lazy`, `Demand`, `Deferred`).
