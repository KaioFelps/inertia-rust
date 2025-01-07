# Installation

To get Inertia working, you'll need to ensure some peer dependencies are installed. Once you opt-in a specific
provider, assure the needed peer crate is also available.

```toml
# Cargo.toml

[dependencies]
inertia-rust = { version = "2", features = ["actix", "vite-template-resolver"] }
actix-web = "4"
vite-rust = { version = "0.2" }
```

The **vite-template-resolver** feature enables the `ViteTemplateResolver`. We'll discuss it furthermore, in the
[Template Resolvers], along with how to set up your own template resolver. On this documentation, we'll use
[vite-rust] and [Actix Web], so you must have them installed.

[Template Resolvers]: ../advanced/template_resolvers.md
[vite-rust]: https://github.com/KaioFelps/vite-rust
[Actix Web]: https://actix.rs/

## Available Crate Features

* `actix`: enable Actix Web provider;
* `vite-template-resolver`: enable `ViteTemplateResolver`.

## Vite Setup

`ViteTemplateResolver` requires a `Vite` instance, so, we need to both set up Vite and Inertia. Read [vite-rust]
docs for more details about setting it up. For this example, consider the following configuration:

```rust
// src/config/vite.rs
use vite_rust::{Vite, ViteConfig};

pub async fn initialize_vite() -> Vite {
    let vite_config = ViteConfig::default()
        .set_manifest_path("path/to/manifest.json")
        // you can add the same client-side entrypoints to vite-rust,
        // so that it won't panic if the manifest file doesn't exist but the
        // development server is running
        .set_entrypoints(vec!["www/app.ts"]);

    match Vite::new(vite_config).await {
        Err(err) => panic!("{}", err),
        Ok(vite) => vite
    }
}
```

[vite-rust]: https://github.com/KaioFelps/vite-rust

## Inertia Setup

```rust
// src/config/inertia.rs
use super::vite::initialize_vite;
use inertia_rust::{
    template_resolvers::ViteTemplateResolver, Inertia, InertiaConfig, InertiaVersion,
};
use std::io;

pub async fn initialize_inertia() -> Result<Inertia, io::Error> {
    let vite = initialize_vite().await;
    let version = vite.get_hash().unwrap_or("development").to_string();
    let resolver = ViteTemplateResolver::new(vite);

    Inertia::new(
        InertiaConfig::builder()
            .set_url("http://localhost:3000")
            .set_version(InertiaVersion::Literal(version))
            .set_template_path("www/root.html")
            .set_template_resolver(Box::new(resolver))
            .build(),
    )
}
```

### Inertia Configuration

| Option            | Type (default)                            | Description   |
| ---               | ---                                       | ---           |
| url               | `&str`                                    | A valid [href](https://developer.mozilla.org/en-US/docs/Web/API/Location) of the current application |
| version           | `InertiaVersion`                          | The current asset version of the application. See [Asset versioning](https://inertiajs.com/asset-versioning) for more details. |
| template_path     | `&str`                                    | The path to the root html template. |
| template_resolver | `Box<dyn TemplateResolver + Send + Sync>` | A valid [Template Resolver]. |
| with_ssr          | `bool` (`false`)                          | Whether Server-side Rendering should be enabled or not. |
| custom_ssr_client | `Option<SsrClient>` (`SsrClient::default`)| The Inertia Server address. |
| view_data         | `Option<ViewData>` (`None`)               | Optional view data to be passed to the root template. (needs to be handled by the Template Resolver) |
| encrypt_history   | `bool` (`false`)                          | Whether to encrypt the session or not. Refer to [History encryption] for more details. |

For even more details, read the [`InertiaConfig`] and [`InertiaConfigBuilder`] documentations.

[Template Resolver]: https://kaiofelps.github.io/inertia-rust/advanced/template_resolvers.html
[Flash Messages and Validation Errors]: https://kaiofelps.github.io/inertia-rust/advanced/flash-messages.html
[History encryption]: https://inertiajs.com/history-encryption
[`InertiaConfig`]: https://docs.rs/inertia-rust/latest/inertia_rust/struct.InertiaConfig.html
[`InertiaConfigBuilder`]: https://docs.rs/inertia-rust/latest/inertia_rust/struct.InertiaConfigBuilder.html

## Actix Web Server
```rust
// src/main.rs
use std::sync::{Arc, OnceLock};
use actix_web::{dev::Path, web::Data, App, HttpServer};
use config::inertia::initialize_inertia;

mod config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    // starts a Inertia manager instance.
    let inertia = initialize_inertia().await?;
    let inertia = Data::new(inertia);

    HttpServer::new(move || App::new().app_data(inertia.clone()))
        .bind(("127.0.0.1", 3000))?
        .run()
        .await
}
```

## Root Template
`ViteTemplateResolver` receives a path to an HTML template file. Inertia Rust will pass the given `template_path`
to the resolver. In this case, it's `www/root.html` file.

```html
<!-- www/root.html -->
<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
    <meta http-equiv="X-UA-Compatible" content="ie=edge">
    <!-- include this if you're using ReactJs -->
    <!-- @vite::react -->
    @vite
    @inertia::head
</head>
<body>
    @inertia::body
</body>
</html>
```

## Inertia Middleware

The Inertia Middleware comes from your opted provider. It has few responsibilities:
- allow you to share props, via `with_shared_props` method (discussed in [Shared Props](../props/shared.md));
- ensure that redirects for `PUT`, `PATCH` and `DELETE` requests always use a 303 status code;
- merge shared props with errors flash messages (discussed in [Flash Messages and Validation Errors](../advanced/flash-messages.md)).

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
            .wrap(InertiaMiddleware::new())
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
```

---

You can check more examples in [Inertia Rust examples] directory. Also, do check [Inertia.js client-side setup] tutorial.

[Inertia Rust examples]: https://github.com/KaioFelps/inertia-rust/tree/v2/examples
[Inertia.js client-side setup]: https://inertia-rails.dev/guide/client-side-setup
