# Deferred Props

Deferred props behave the exactly same way as `InertiaProp::Demand`. However, when you use `Deferred` instead of
`Demand`, Inertia router will automatically make a partial request to obtain the deferred props after the page
loads.

It means deferred data will be loaded after the page has already been rendered, so if you defer heavy or time-
consuming data, your page might load faster.

## Deferring a Prop

```rust
use actix_web::{get, HttpRequest, Responder};
use inertia_rust::{hashmap, prop_resolver, Inertia, InertiaProp, IntoInertiaPropResult};

// let's pretend these are some ORM's models and
// assume they have an `all` method.
use domain::identity::models::Role;
use domain::identity::models::User;
use domain::identity::models::Permission;

#[get("/users")]
pub async fn users(req: HttpRequest) -> impl Responder {
    Inertia::render_with_props(&req, "Users/Index", hashmap![
        'users' => InertiaProp::data(User::all().await),
        'roles' => InertiaProp::data(Role::all().await),
        'permissions' => InertiaProp::defer(prop_resolver!({ Permission::all().await.into_inertia_value() })),
    ]).await
}
```

## Grouping Requests

Deferred prop are splitten in groups. Each group is paralelly requested by the client Inertia router.
`InertiaProp::defer` will put the resolver in the `"default"` group of deferred props. If you want to
create a custom group, call `InertiaProp::defer_with_group` passing the group name as the second parameter:

```rust
#[get("/users")]
pub async fn users(req: HttpRequest) -> impl Responder {
    Inertia::render(&req, "Users/Index", hashmap![
        'users' => User::all(),
        'roles' => Role::all(),
        'permissions' => InertiaProp::defer(prop_resolver!({ Permission::all().await.into_inertia_value() })),
        'teams' => InertiaProp::defer_with_group(prop_resolver!({ Team::all().await.into_inertia_value() }), 'attributes'),
        'projects' => InertiaProp::defer_with_group(prop_resolver!({ Project::all().await.into_inertia_value() }), 'attributes'),
        'tasks' => InertiaProp::defer_with_group(prop_resolver!({ Task::all().await.into_inertia_value() }), 'attributes'),
    ]).await
}
```
