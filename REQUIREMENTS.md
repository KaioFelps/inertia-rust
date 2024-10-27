# Requirements of an Inertia.js adapter

## Responses and routing
- [x] Return a rendered Inertia Page as Http Response;

render a page; <br/>
pass props to the front-end

```rust
#[derive(Serialize, Deserialize)]
struct User { ... }

let mut props = HashMap::new();
props.insert("user".into(), serde_json::to_value(user).unwrap());
props.insert("title".into(), format!("Editing {}", user.name);

// return inertia_rust::render<T>(&req, "User"); // render without props
return inertia_rust::render_with_props::<T>(&req, Component("Users/Index".into()), props)
        .await
        .unwrap_or(HttpResponse::InternalServerError().finish());
```

---

- [ ] Render response with `view data`;

View Data are data that will be passed for the root template (e.g. handlebars)
```rust
struct EventProps {
   event: Event   
}

struct RootProps {
    meta: MetaDataInfos
}

let props = EventProps {
    event,
}

// this data will only be accessible to the root template
let rootData = RootProps {
    meta,
}

Inertia::render("Event", props).with_view_data(rootProps)
```

---

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

To render a inertia page with errors, call the method `Inertia::render(...).with_errors()` or render the page with a
custom props struct that contains an `error` field.
```rust
struct ErrorsStruct {
    name: Option<SomeValidationError>,
    age: Option<SomeValidationError>
}

let errors = ErrorsStruct {
    name: None,
    age: Some(SomeValidationError::from("Invalid age, for some reason".to_string()))
};

// return Inertia::render("Index").with_errors(errors); // renders a page with errors
return Inertia::redirect("/").with_errors(errors); // redirects to a page with errors

// ===
// or with custom props struct
// ===

#[derive(Serialize, Deserialize)]
struct CustomPropsStruct {
    AnyOtherProperty: String,
    
    #[serde(rename = "errors")] // note that field name must always be lowercase. Inertia's requirement...
    Errors: Option<ErrorsStruct>
}

let props = CustomPropsStruct {
    AnyOtherProperty: "Foo".into(),
    Errors: Some(errors)
};

return Inertia::redirect("/", props);
```

## Title & Meta

- [ ] Inject the inertia's Head component content inside a markup at the root template with a
middleware or some preprocessor

## Shared Data
- [ ] Middleware that merges global shared data with page shared data.
```rust
use actix_web::App;
use serde_json::{Map, Value};
use serde::{Deserialize, Serialize};

// server setup...
App::new()
    // returns a middleware that shares these values globally.
    // PageProps field holds the props available for the rendered page only.
    .wrap(Inertia::SharedPropsMiddleware::share(|props: &mut Map<String, Value>| {
        #[derive(Serialize)]
        struct User {
            nickname: String,
            active: bool
        }
        
        let user = User {
            nickname: "John".into(),
            active: true,
        };
        
        let mut auth_map = Map::new();
        auth_map.insert("user".into(), serde_json::to_value(user).unwrap());
        
        props.insert("foo".into(), Value::String("this is a global value!".into()));
        props.insert("bar".into(), 255.into());
        props.insert("auth".into(), auth_map.into());
    }))
```
This would output something like:
```rust
{
    "auth": Object {
        "user": Object {
            "active": Bool(true),
            "nickname": String("John"),
        },
    },
    "bar": Number(255),
    "foo": String("this is a global value!"),
    // page props, had been already set to `props` map
    "event": Object {
        "max": Number(8),
        "cancelled": Bool(false),
    },
}
```