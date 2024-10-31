# Inertia Rust

A server-side Inertia.js adapter for Rust. Inertia Rust aims to interoperate with any
Rust (micro-)framework and template engine, since a compatible provider exists.

Due to its flexibility, it requires a bit more configuration. Hence, please read
this document carefully to ensure that your Inertia application works correctly.

## Getting started

```toml
[dependencies]
inertia_rust = { version = "0.1", features = ["default", "basic-vite-resolver"] }
actix-web = "4"
vite-rust = { version = "0.1", features = ["basic_directives"] }
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
use std::sync::OnceLock;
use vite_rust::{Vite, ViteConfig};
use inertia_rust::{Inertia, InertiaVersion};

static VITE: OnceLock<Vite> = OnceLock::new();

fn main() -> std::io::Result<()> {
    // initializes Vite
    let vite_config = ViteConfig::new_with_defaults("public/bundle/manifest.json");
    let vite = match Vite::new(vite_config).await {
        Ok(vite) => vite,
        Err(err) => panic!("{}", err)
    };
    
    // initializes Inertia store
    let inertia = Inertia::new(
        "http://localhost:8080",
        // InertiaVersion::Literal(vite.get_hash()), or
        InertiaVersion::Resolver(Box::new(|| vite.get_hash())),
        "path/to/your/template.html",
        &inertia_rust::template_resolvers::basic_vite_resolver,
        vite
    );
    
    // stores Inertia as an AppData in a way that is not cloned
    let inertia = Data::new(inertia);
    let inertia_clone = Data::clone(&inertia);

    HttpServer::new(move || { App::new().app_data(inertia_clone.clone()) })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await;
}
```

#### Server-side rendering

If you have Node.js available in the machine your Rust application is running at, you can enable
**server-side rendering**. For this, you'll need to do some few changes in your code:

```rust
use std::sync::OnceLock;
use vite_rust::{utils::resolve_path, Vite, ViteConfig};
use inertia_rust::{Inertia, InertiaVersion, SsrClient};

static VITE: OnceLock<Vite> = OnceLock::new();

fn main() -> std::io::Result<()> {
    // initializes Vite
    let vite_config = ViteConfig::new_with_defaults("public/bundle/manifest.json");
    let vite = match Vite::new(vite_config).await {
        Ok(vite) => vite,
        Err(err) => panic!("{}", err)
    };

    // initializes Inertia store
    let inertia: Inertia<Vite> = Inertia::new_with_ssr(
        "http://localhost:8080", // url of 
        InertiaVersion::Literal(vite.get_hash().to_string()),
        "path/to/your/template.html",
        &inertia_rust::template_resolvers::basic_vite_resolver,
        vite,
        Some(SsrClient::new("127.0.0.1", 1000))
    ).await?;

    // stores Inertia as an AppData in a way that is not cloned
    let inertia = Data::new(inertia);
    let inertia_clone = Data::clone(&inertia);

    let server = HttpServer::new(move || {
        App::new().app_data(inertia_clone.clone())
    }).bind(("127.0.0.1", 8080))?;

    // Starts a Node.js child process that runs the Inertia's server-side-rendering server.
    // It must be started after the server initialization to ensure that the server won't panic and
    // shutdown without killing Node process.
    let node = inertia.start_node_server("path/to/your/ssr.js".into())?;

    let server = server.run().await;
    let _ = node.kill();
    
    return server;
}
```

## Pages rendering
*Still under development...*

---

[> [!WARNING]
> This is in the very first stages of development. A list of functionalities to be
> implemented is available in the <a href="./REQUIREMENTS.md">REQUIREMENTS</a> file.
> The below list shows what have already been implemented so far.
> - [x] Inertia page rendering with or without props.
]()