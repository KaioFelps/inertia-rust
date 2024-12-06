# Inertia Rust

A server-side Inertia.js adapter for Rust. Inertia Rust aims to interoperate with any
Rust (micro-)framework and template engine, since a compatible provider exists.

Due to its flexibility, it requires a bit more configuration. Hence, please read
this document carefully to ensure that your Inertia application works correctly.

## Getting started

```toml
[dependencies]
inertia-rust = { version = "0.1", features = ["default", "basic-vite-resolver"] }
actix-web = "4"
vite-rust = { version = "0.2", features = ["basic-directives"] }
```

To get Inertia working, you'll need to ensure some peer dependencies are installed.
Currently, inertia_rust is still under development and is working to support Actix Web.
Therefore, ensure you have actix_web on your dependency section and "default" or "actix"
feature is enabled at inertia_rust dependency properties.

"basic-vite-resolver" feature enables few basic `vite-rust` directives. Currently, we
still do not have default support for template engines, even though you can easily set
it up by yourself.

The basic vite resolver is a `template-resolver` function that uses (and thus require you
to install) `vite-rust` basic directives (also must be enabled manually at features property)
to inject Vite's tags and Inertia's body and head into your HTML template.

### Creating your own template resolver
A template resolver function must be provided during Inertia setup. The basic vite resolver
might fit for most usages. If you need something more specific, you will need to create
two functions: one to actually resolve the template and a wrapper function, so that the resolver
can be stored inside Inertia structure.

```rust
use inertia_rust::{InertiaError, TemplateResolverOutput, ViewData};

// the actual resolver
async fn resolver(
    path: &str,
    view_data: ViewData,
    some_useful_prop: &SomeUsefulStruct
) -> Result<String, InertiaError> {
    /* ... */
}

// a function that wraps the resolver
pub fn template_resolver(
    template_path: &'static str,
    view_data: ViewData,
    prop: &'static SomeUsefulStruct
) -> TemplateResolverOutput {
    Box::pin(resolver(template_path, view_data, prop))
}
```

You might have noted that the third parameter is a reference to `SomeUsefulStruct`. This must be
some useful struct used by your resolver. For instance, our basic vite resolver requires a
static reference to a `vite_rust::Vite` struct, because it's what provides the HTML tags of the modules,
HMR and other important stuff that must be injected into the HTML.

This struct will also be stored by static reference inside Inertia struct, and Inertia is the one who will
call the resolver method when rendering your HTTP response.

If you don't need any extern struct, you can simply pass a `&'static ()` on Inertia's `template_resolver_data`
field. Note that, *Inertia<T>* requires *template_resolver*'s third parameter to be of type *T* either.

### Inertia setup

For this guide, I'll consider you're using `vite-rust` and `actix-web`, with the above Cargo.toml dependencies.
Inside your `main.rs`, you'll have to:

1. Declare Vite as a static constant;
2. Initialize Vite;
3. Initialize Inertia with a static reference to your Vite instance.

```rust
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use inertia_rust::resolvers::basic_vite_resolver;
use inertia_rust::{Inertia, InertiaConfig, InertiaVersion};
use std::sync::OnceLock;
use vite_rust::{Vite, ViteConfig};

static VITE: OnceLock<Vite> = OnceLock::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // initializes Vite
    let vite = match Vite::new(
        ViteConfig::default()
            .set_manifest_path("public/bundle/manifest.json")
            .set_entrypoints(vec!["app.ts"]),
    )
    .await
    {
        Ok(vite) => vite,
        Err(err) => panic!("{}", err),
    };

    let vite = VITE.get_or_init(move || vite);

    let inertia_config: InertiaConfig<Vite, String> = InertiaConfig::builder()
        .set_url("http://localhost:3000")
        // InertiaVersion::Literal(vite.get_hash()), or
        .set_version(InertiaVersion::Literal(
            vite.get_hash()
                .map(ToString::to_string)
                .unwrap_or("development-version".into()),
        ))
        .set_template_path("path/to/your/template.html")
        .set_template_resolver(&basic_vite_resolver)
        .set_template_resolver_data(vite)
        .build();

    // initializes Inertia struct
    let inertia = Inertia::new(inertia_config)?;

    // stores Inertia as an AppData in a way that is not cloned for each worker
    let inertia = Data::new(inertia);
    let inertia_clone = Data::clone(&inertia);

    HttpServer::new(move || App::new().app_data(inertia_clone.clone()))
        .bind(("127.0.0.1", 3000))?
        .run()
        .await
}
```

#### Server-side rendering

If you have Node.js available in the machine your Rust application is running at, you can enable
**server-side rendering**. For this, you'll need to do some few changes in your code:

```rust
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use inertia_rust::resolvers::basic_vite_resolver;
use inertia_rust::{Inertia, InertiaConfig, InertiaVersion, SsrClient};
use std::sync::OnceLock;
use vite_rust::{Vite, ViteConfig};

static VITE: OnceLock<Vite> = OnceLock::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // initializes Vite
    let vite = match Vite::new(
        ViteConfig::default()
            .set_manifest_path("public/bundle/manifest.json")
            .set_entrypoints(vec!["app.ts"]),
    )
    .await
    {
        Ok(vite) => vite,
        Err(err) => panic!("{}", err),
    };

    let vite = VITE.get_or_init(move || vite);

    let inertia_config: InertiaConfig<Vite, String> = InertiaConfig::builder()
        .set_url("http://localhost:3000")
        // InertiaVersion::Literal(vite.get_hash()), or
        .set_version(InertiaVersion::Literal(
            vite.get_hash()
                .map(ToString::to_string)
                .unwrap_or("development-version".into()),
        ))
        .set_template_path("path/to/your/template.html")
        .set_template_resolver(&basic_vite_resolver)
        .set_template_resolver_data(vite)
        .enable_ssr()
        // `set_ssr_client` is optional. If not set, `SsrClient::default()` will be used.
        .set_ssr_client(SsrClient::new("127.0.0.1", 1000))
        .build();

    // initializes Inertia struct
    let inertia = Inertia::new(inertia_config)?;

    // stores Inertia as an AppData in a way that is not cloned
    let inertia = Data::new(inertia);
    let inertia_clone = Data::clone(&inertia);

    let server = HttpServer::new(move || App::new().app_data(inertia_clone.clone()))
        .bind(("127.0.0.1", 3000))?;

    // Starts a Node.js child process that runs the Inertia's server-side-rendering server.
    // It must be started after the server initialization to ensure that the server won't panic and
    // shutdown without killing Node process.
    let node = inertia.start_node_server("path/to/your/ssr.js".into())?;

    let server = server.run().await;
    let _ = node.kill().await;

    return server;
}
```

## Page rendering and Responses
There are a few couple ways of rendering an Inertia page. Every provider will aim to give you
as many facilities as possible.

The following application renders a component "Index" without any props at "/" endpoint.
```rust
use actix_web::{get, App, HttpRequest, HttpServer, Responder};
use inertia_rust::{actix::render, InertiaErrMapper};

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    render::<Vite>(&req, "Index".into()).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ...
    
    HttpServer::new(move || {
        App::new()
            .app_data(inertia_clone.clone())
            .service(index)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
```

The very same thing could be done in the following way:
```rust
use actix_web::{App, HttpServer};
use inertia_rust::InertiaService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ...
    
    HttpServer::new(move || {
        App::new()
            .app_data(inertia_clone.clone())
            .inertia_route::<Vite>("/", "Index")
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
```

However, it is not possible to render pages with props using `inertia_route` method. It must
be done with an ordinary handler function and with `render_with_props` helper:

```rust
use actix_web::{get, App, HttpRequest, HttpServer, Responder};
use inertia_rust::{actix::render_with_props, InertiaErrMapper, InertiaProp};
use std::collections::HashMap;

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    let mut props = HashMap::new();
    props.insert(
        "message".to_string(),
        InertiaProp::Always("Hello world!".into()),
    );

    render_with_props::<Vite>(&req, "Index".into(), props).await
}

// ...
```

An `InertiaProp` is an enum that can hold a `serde_json::Value` or a callback that returns one of it.
A hash map of InertiaProp elements is an `InertiaProps` set, and it's resolved during rendering (when
the needed props are evaluated).

## Inertia Middleware and Shared Props

The Inertia Middleware comes from your opted provider. It has few responsibilities:
- allow you to **share props**, via `with_shared_props` method;
- ensure that redirects for PUT, PATCH and DELETE requests always use a 303 status code;
- merge shared props with errors flash messages.

The middleware's `with_shared_props` method requires a callback, wrapped in an `Arc`, that
receives a reference to the current request. You can use it to extract any information you might want
to share.

```rust
use actix_web::{App, HttpServer};
use inertia_rust::actix::InertiaMiddleware;
use inertia_rust::{InertiaProp, InertiaProps, InertiaService};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ...

    HttpServer::new(move || {
        App::new()
            .app_data(inertia_clone.clone())
            .wrap(InertiaMiddleware::new().with_shared_props(Arc::new(|req| {
                let mut props: InertiaProps = HashMap::<String, InertiaProp>::new();

                // get the sessions from the request
                // depending on your opted framework
                let session = req.get_session();
                let flash = serde_json::to_value(session.get::<String>("flash").unwrap()).unwrap();

                props.insert("flash".to_string(), InertiaProp::Always(flash));
                props
            })))
            .inertia_route::<Vite>("/", "Index")
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
```

It will also flash errors into the shared props. These props will be injected back into the request
and will be further merged again with the page props during rendering, thus making all of them
available to your client-side page component.

As inertia-rust is not made for one single framework and any of them actually have built-in sessions
management, you need to built by yourself a second middleware that injects in the request
context/extensions an `InertiaTemporarySession` object:

```rust
#[derive(Clone, Serialize)]
pub struct InertiaTemporarySession {
    // Optional errors hashmap
    pub errors: Option<Map<String, Value>>,
    // The previous request URL
    // useful for redirecting back with errors
    pub prev_req_url: String,
}
```
Inertia Middleware tries to extract this from the request context and merge it with the shared
props. This is how validation errors get available to your page components.

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
            
                let error = session
                    .remove("error")
                    .map(|error| serde_json::from_str(&error).unwrap());
            
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
                    error,
                    prev_req_url: prev_url,
                };
            
                req.extensions_mut().insert(temporary_session);
            
                let fut = service.call(req);
                async {
                    let res = fut.await?;
                    Ok(res)
                }
            })
            .wrap(SessionMiddleware::new(...))
            .wrap(InertiaMiddleware::new())
            .inertia_route::<Vite>("/", "Index")
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
```

Yet you need to enable your framework session middleware and manager (or your own). As errors
are retrieved by `remove` method, they are only available for one request lifetime. Errors and
flash messages shouldn't persist across multiple requests.

---

> [!WARNING]
> This is in the very first stages of development. A list of functionalities to be
> implemented and what have been so far is available in the
> <a href="./REQUIREMENTS.md">REQUIREMENTS</a> file.
