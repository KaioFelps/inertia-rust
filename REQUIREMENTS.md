# Requirements of an Inertia.js adapter

## Responses and routing
- [x] Render an Inertia Page (with or without props);

```rust
#[derive(Serialize, Deserialize)]
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

---

- [x] ~~Render response with `view data`~~;

View Data are data that will be passed for the root template (e.g. handlebars).
Inertia's `ViewData` struct accepts an optional HashMap with custom values inside it. The resolver
can be used to consume this hashmap and insert props.

- [ ] Shorthand renderer for routes without handlers;

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
  HttpServer::new(|| {
    App::new()
            
    .route("/", Inertia::route("Home"))
    // or using actix_web inertia extension
    .inertia("/", "Home")
    
  })
  .bind(("127.0.0.1", 8080))?
  .run()
  .await
}
```

- [ ] Render / Redirect with errors.

Some ways of rendering with errors are:
- [x] Render the errors as props;
- [ ] Redirect back to the previous URL with the errors as flash messages (and
  let the Inertia Middleware merge them into the props by itself).
```rust
use std::collections::hash_map::HashMap;
use serde_json::json;
use inertia_rust::Component;
use vite_rust::Vite;
use some_framework::{SomeHttpRequest, SomeHttpResponse, Redirect};

async fn some_handler(req: SomeHttpRequest) -> SomeHttpResponse {
  let mut props = HashMap::<String, serde_json::Value>::new();
  props.insert("errors", json!{
      "age": serde_json::to_value("Invalid age, for some reason".to_string()).unwrap(),
  });
  
  return inertia_rust::render_with_props::<Vite>(&req, Component("Contact".into()), props)
      .await
      .map_inertia_err();
}

async fn another_handler(req: SomeHttpRequest) -> SomeHttpResponse {
  // A framework built-in redirect to the previous URL.
  // The error should be stored in a session (also provided by the framework)
  // and further injected in the props by the Inertia Middleware on the
  // subsequent request.
  return Redirect::back()
    .add_session("errors".to_string(), json!({
        "age": serde_json::to_value("Invalid age, for some reason".to_string()).unwrap(),
    }))
    .finish();
}
```

## Assets Versioning
When the assets versions mismatch, the rendering method should return an
`Inertia::location` redirect that causes a full-reload.

- [ ] Forward the session to be retrieved by the request triggered by the reload.

## Inertia Middleware
- [ ] Allow to **share props** globally;
- [ ] Convert Redirect requests;
- [ ] Merge error and flash messages from Session into the page props.