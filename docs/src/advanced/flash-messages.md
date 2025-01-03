# Flash Messages and Validation Errors

[Inertia Middleware](../basic/installation.md#inertia-middleware) will also merge flash errors with the
shared props. The resulting props will be injected back into the request and will be further merged again
with the page props during rendering, thus, making all of them available to your client-side page component.

As said earlier, Inertia Rust is not made for one single framework and all of them actually might have
built-in sessions management. Hence, you need to built by yourself a second middleware that injects an `InertiaTemporarySession` object in the request context/extensions:

```rust
// InertiaTemporarySession struct from Inertia Rust
#[derive(Clone, Serialize)]
pub struct InertiaTemporarySession {
    // Optional errors hashmap
    pub errors: Option<Map<String, Value>>,
    // The previous request URL
    // useful for redirecting back with errors
    pub prev_req_url: String,
}
```

The middleware tries to extract this from the request context and merge it with the shared props. This
is how validation errors might get available to your page components without explicitly sending them with `Inertia::render_with_props`.

## Temporary Session Middleware

Check a sample middleware that extracts errors from the session and add to extensions:

```rust
use inertia_rust::{InertiaTemporarySession, InertiaMiddleware};
use actix_session::{SessionExt, SessionMiddleware};
use actix_web::{dev::Service, App, HttpMessage, HttpServer};
use serde_json::{Map, Value};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap_fn(|req, service| {
                const PREV_REQ_KEY: &str = "_prev_req_url";
                const CURR_REQ_KEY: &str = "_curr_req_url";
                
                let session = req.get_session();
            
                let errors = session
                    .remove("_errors")
                    .map(|errors| serde_json::from_str(&errors).unwrap());
            
                // gets the previous request's URI and stores the current one's,
                // so that it becomes the previous request URI of the next request.
                // ---
                
                let prev_url = session
                    .get::<String>(CURR_REQ_KEY)
                    .unwrap_or(None)
                    .unwrap_or("/".to_string());
            
                if let Err(err) = session.insert(PREV_REQ_KEY, &prev_url) {
                    eprintln!("Failed to update session previous request URL: {}", err);
                };
            
                if let Err(err) = session.insert(CURR_REQ_KEY, req.uri().to_string()) {
                    eprintln!("Failed to update session current request URL: {}", err);
                };
                
                // ---
            
                let temporary_session = InertiaTemporarySession {
                    errors,
                    prev_req_url: prev_url,
                };
            
                req.extensions_mut().insert(temporary_session);
            
                let fut = service.call(req);
                async {
                    let res = fut.await?;
                    Ok(res)
                }
            })
            .wrap(SessionMiddleware::new(/*...*/))
            .wrap(InertiaMiddleware::new())
            .inertia_route("/", "Index")
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
```

Yet you need to enable your framework session middleware and manager (or your own). As errors
are retrieved by `remove` method, they are **only available for one request lifetime**. Indeed,
errors and flash messages shouldn't persist across multiple requests.
