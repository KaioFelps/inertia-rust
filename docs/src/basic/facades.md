# The Facades

Inertia Rust has several traits that describe its behavior. The provider's job is to implement them
to the framework structs, connecting Inertia Rust to it.

The main trait describes the operations `inner_render`, `inner_render_with_props` and `inner_location`,
which must be implemented and contains the actual methods (the implementations of the operations).
For using these functions, you'd have to extract the `Inertia` instance from the request context and,
again, pass a reference to the whole http request to it.

It's definetely far from being fancy. That's why we also provide --- and require provides to implement
--- the `InertiaFacade` trait. It contains the `render`, `render_with_props` and `location` operations.
The provider uses the given request reference to extract the `Inertia` instance itself and call the
respective method.

It's way more elegant to do this:

```rust
use inertia_rust::{Inertia, InertiaFacade};
use actix_web::{get, Responder, HttpRequest};

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    Inertia::render(&req, "Index".into()).await
}
```

Than this:

```rust
use inertia_rust::Inertia;
use actix_web::{get, Responder, HttpRequest, web::Data};

#[get("/")]
async fn index(req: HttpRequest, inertia: Data<Inertia>) -> impl Responder {
    inertia.inner_render(&req, "https://inertiajs.com").await
}
```
