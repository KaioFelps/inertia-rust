# Requirements of an Inertia.js adapter

## Responses and routing
- [x] Render an Inertia Page (with or without props);

```rust
use std::collections::HashMap;
use serde_json::json;
use some_framework::{SomeHttpRequest, SomeHttpResponse, Redirect};
use serde::{Serialize, Deserialize};
use vite_rust::Vite;
use inertia_rust::Component;

#[derive(Serialize, Deserialize)]
struct User {
  name: String
}

async fn some_handler(req: SomeHttpRequest) -> SomeHttpResponse {
  let user = User {
    name: "John Doe"
  };
  
  let mut props = HashMap::new();
  props.insert("user".into(), serde_json::to_value(&user).unwrap());
  props.insert("title".into(), format!("Editing {}", user.name));

  // return inertia_rust::render::<Vite>(&req, Component("Users/Index".into()))
  // or
  return inertia_rust::render_with_props::<Vite>(&req, Component("Users/Index".into()), props)
    .await
    .map_inertia_err();
}
```

- [x] Shorthand renderer for routes without handlers;

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
  HttpServer::new(|| {
    App::new()
      .inertia_route("/", "Home")
    
  })
  .bind(("127.0.0.1", 8080))?
  .run()
  .await
}
```

---

## Rendering response with `view data`

ViewData holds data that will be passed for the root template.
Inertia's `ViewData` struct accepts an optional HashMap with custom values inside it.
The resolver must be responsible by allowing the template to consume this
hashmap values.

---

## Render / Redirect with errors.

Some ways of rendering with errors are:
- [x] Render the errors as props;
- [x] Redirect back to the previous URL with the errors as flash messages (and
  let the Inertia Middleware merge them into the props by itself).
  - Must be partially implemented by the dev using inertia-rust.

```rust
use std::collections::hash_map::HashMap;
use serde_json::json;
use inertia_rust::InertiaTemporarySession;
use vite_rust::Vite;
use some_framework::{SomeHttpRequest, SomeHttpResponse, Redirect};

async fn some_handler(req: SomeHttpRequest) -> SomeHttpResponse {
  let mut props = HashMap::<String, serde_json::Value>::new();
  props.insert(
    "errors",
    json!({ "age": "Invalid age, for some reason." })
  );
  
  return inertia_rust::render_with_props::<Vite>(&req, "Contact".into(), props)
      .await
      .map_inertia_err();
}

async fn another_handler(req: SomeHttpRequest, inertia_session: InertiaTemporarySession) -> SomeHttpResponse {
  // A framework built-in redirect to the previous URL.
  // The error should be stored in a session (also provided by the framework).
  // It will be merged with the next response props by Inertia Middleware.
  req.add_to_session("errors", json!({"age": "Invalid age, for some reason."}));
  Redirect::to(inertia_session.prev_req_url).with_status(303)
}
```

## Assets Versioning
When the assets versions mismatch, the rendering method should return an
`Inertia::location` redirect that causes a full-reload.

**Solution**
> Inertia requires you to pass callback during Inertia configuration
> that is responsible for reflashing the inertia temporary session.

- [x] Forward the session to be retrieved by the request triggered by the reload.

## Inertia Middleware
- [x] Allow to **share props** globally;
- [x] Convert Redirect requests;
- [x] Merge error from Session into the page props.