# Merging Props

Merging Props are merged with the existing props in the client-side, instead of overwriting them. For instance,
if your client already has a property `"permissions"` that is a list `["read", "delete"]`, given that
`"permissions"` is a mergeable prop, when a new partial request receives a list `"permissions" = ["update"]`,
the new value of `"permissions"` in the client will be `["read", "delete", "update]` instead of `["update"]`.

## Making a Mergeable Prop

```rust
use actix_web::{web, get, HttpRequest, Responder};
use inertia_rust::{hashmap, Inertia, InertiaProp};
use serde::Deserialize;

#[derive(Deserialize)]
struct RequestQuery {
    page: Option<usize>,
    per_page: Option<usize>
}

#[get("/users")]
pub async fn users(req: HttpRequest, query: web::Query<RequestQuery>) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);

    Inertia::render_with_props(&req, "Users/Index", hashmap![
        'results' => InertiaProp::merge(User::paginate(page, per_page).await),
    ])
}
```

Again, let's pretend `User` is a model of some ORM which contains a `paginate` method.

**Deferred props can also be mergeable**. The behavior will be a combination of them both: the deferred prop
will be fetched in a separate request, and when it is received, it'll be merged with the existing version of itself.

```rust
#[get("/users")]
pub async fn users(req: HttpRequest, query: web::Query<RequestQuery>) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);

    return Inertia::render_with_props(&req, "Users/Index", hashmap![
        'results' => InertiaProp::defer(prop_resolve!({ User::paginate(page, per_page).await })).into_mergeable(),
    ])
}
```

## Resetting Props

When a mergeable property is asked to be reset, it will behave like an ordinary `InertiaProp::Data` variant
--- or a `InertiaProp::Deferred`, if it's a deferred and mergeable prop.

```js
router.reload({
    reset: ['results'],
    //...
})
```