# Props

It's possible to pass data from the server to the front-end as simple props. To do this, simply
create a `HashMap` with `&str` keys and `InertiaProp` values. Then, use `Inertia::render_with_props`
passing the props hashmap as the third parameter.

Also, for defining the props hash map, we provide a very trivial macro: `hashmap!`. You can use it
like this:

```rust
use inertia_rust::{hashmap, Inertia, InertiaFacade, InertiaProp};
use actix_web::{get, web, HttpRequest, Responder};
use serde_json::{json};
use serde::Serialize;

#[derive(Serialize)]
struct User {
    pub name: String,
    pub email: String,
}

impl User {
    // let's just pretend this is a method that fetch users from some database
    pub async fn all() -> Vec<User> {
        vec![
            User {
                name: "John Doe".into(),
                email: "johndoe@gmail.com".into()
            },
            User {
                name: "Ada Lovelace".into(),
                email: "thereWereNoEmailByThatTimeActually!@gmail.com".into(),
            }
        ]
    }
}

#[get("/users")]
async fn home(req: HttpRequest) -> impl Responder {
    let props = hashmap![
        "users" => InertiaProp::data(User::all().await),
    ];

    Inertia::render_with_props(&req, "Users/Index".into(), props).await
}
```

Indeed, all `hashmap!` do is to create a `std::collections::HashMap` and insert each item you've passed
into it.

Note that a new enum has been introduced to our code right now: `InertiaProp`. It contains all the variants
of props that Inertia can handle.

Every `InertiaProp` variant must contain --- or resolve to --- an `Result<serde_json::Value, InertiaError`>
object. You can achieve this by calling `value.into_inertia_value()`. `value` must be serializable. Also,
you must bring the `IntoInertiaPropResult` trait into the scope.

If you're using `InertiaProp` helpers, you don't even need to worry about serializing by yourself (again,
since the object *is* serializable):

```rust
use inertia_rust::{InertiaProp, IntoInertiaPropResult};

let prop = InertiaProp::Data("foo".into_inertia_value());
// or
let prop = InertiaProp::data("foo"); // using `data` helper for generating a `InertiaProp::Data`
```

On the official Inertia.js documentation, you may find the following PHP snippet:

```php
return Inertia::render('Users/Index', [
    // ALWAYS included on standard visits
    // OPTIONALLY included on partial reloads
    // ALWAYS evaluated
    'users' => User::all(),

    // ALWAYS included on standard visits
    // OPTIONALLY included on partial reloads
    // ONLY evaluated when needed
    'users' => fn () => User::all(),

    // NEVER included on standard visits
    // OPTIONALLY included on partial reloads
    // ONLY evaluated when needed
    'users' => Inertia::lazy(fn () => User::all()),

    // ALWAYS included on standard visits
    // ALWAYS included on partial reloads
    // ALWAYS evaluated
    'users' => Inertia::always(User::all()),
]);
```

You can achieve the exact same behaviors with Inertia Rust:

```rust
return Inertia::render_with_props(&req, "Users/Index", hashmap![
    // ALWAYS included on standard visits
    // OPTIONALLY included on partial reloads
    // ALWAYS evaluated
    "users" => InertiaProp::data(User::all().await),

    // ALWAYS included on standard visits
    // OPTIONALLY included on partial reloads
    // ONLY evaluated when needed
    "users" => InertiaProp::lazy(prop_resolver!({ User::all().await.into_inertia_value() })),

    // NEVER included on standard visits
    // OPTIONALLY included on partial reloads
    // ONLY evaluated when needed
    "users" => InertiaProp::demand(prop_resolver!({ User::all().await.into_inertia_value() })),

    // ALWAYS included on standard visits
    // ALWAYS included on partial reloads
    // ALWAYS evaluated
    "users" => InertiaProp::always(User::all().await),
]);
```

## The `prop_resolver!` macro

Inertia props that take callbacks --- such as `Lazy`, `Demand` and `Deferred` variants --- are
asynchronous, so that you can `.await` inside of them. To use it, it'd be necessary the following code:

```rust
use std::sync::Arc;
use inertia_rust::InertiaProp;
use inertia_rust::IntoInertiaPropResult;

let lazy_prop = InertiaProp::Lazy(Arc::new(move || Box::pin(async move {
    User::all().await.into_inertia_value();
})));
```

Such a massive amount of code, we know. However, it's necessary in order to have asynchronous callbacks
support.

In order to avoid writting all this boilerplate, Inertia Rust provide another macro: `prop_resolver!`,
as you've seen above:

```rust
use std::sync::Arc;
use inertia_rust::{prop_resolver, InertiaProp, IntoInertiaPropResult};

let lazy_prop = InertiaProp::Lazy(prop_resolver!({ User::all().await.into_inertia_value() }));
```

There are some cases --- mainly in test environments or playgrounds --- where you might want to mock
some database and, therefore, need to move values to inside of the resolver closure:

```rust
use std::sync::Arc;
use inertia_rust::InertiaProp;
use inertia_rust::IntoInertiaPropResult;

let user = Arc::new(User { name: "John Doe".into(), email: "johndoe@gmail.com".into() });
let permissions = Arc::new(vec!["read", "delete", "update", "delete"]);

InertiaProp::lazy(Arc::new(move || {
    let user = user.clone();
    let permissions = permissions.clone();

    Box::pin(async move {
        user_can(user, permissions).await.into_inertia_value()
    })
}));
```

Unfortunately, we have to move a clone of the `Arc`-wrapped variables to inside of the closure,
then move again to inside of the inner async closure.

`prop_resolver!` also covers this. As first parameter, you'll pass the "cloning" statements separated
by commas. The actual async closure goes as the second parameter, then:

```rust
use std::sync::Arc;
use inertia_rust::{prop_resolver, InertiaProp, IntoInertiaPropResult};

let user = Arc::new(User { name: "John Doe".into(), email: "johndoe@gmail.com".into() });
let permissions = Arc::new(vec!["read", "delete", "update", "delete"]);

InertiaProp::lazy(prop_resolver!(
    let user = user.clone(),                // statements separated by comma
    let permissions = permissions.clone();  // statements and block separated by semicolon
    { user_can(user, permissions).await.into_inertia_value() }   // block
));
```
