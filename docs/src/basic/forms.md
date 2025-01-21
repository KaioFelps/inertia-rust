# Forms

For validating forms, you need to use some validator crate, such as [garde] or [validator].
However, whereas in a regular API you would simply return the validation errors, when building an
Inertia application, every response must be a valid *Inertia response*.

This means it's necessary to parse the validation errors and return back to the referer URL with
the errors.

> NOTE: in order to work with error flash messages, you *need* to configure your
> [Reflash Session Middleware] properly. Also, you need to configure your framework sessions
> manager.

As this is something that happens very often, we offer a baked-in helper for parsing errors.
Currently, the helper is only available for [validator] along with actix web. You can enable it
through the `actix-validator` feature flag.

> Indeed, we have plans to provide integrations with `garde` and other frameworks in the future.

```toml
# Cargo.toml
[dependencies]
inertia-rust = { version = "2", features = ["actix-validator", ...] }
validator = "0.20"
# ...
```

[garde]: https://crates.io/crates/garde
[validator]: https://crates.io/crates/validator
[Reflash Session Middleware]: ../advanced/flash-messages.md

```rust
use actix_web::{get, post, Responder, HttpRequest, Redirect};
use inertia_rust::{hashmap, Inertia, InertiaFacade, InertiaProp};
use inertia_rust::validators::InertiaValidateOrRedirect;
use validator::Validator;
use serde::Deserialize;

#[get("/users")]
async fn index(req: HttpRequest) -> impl Responder {
    Inertia::render(&req, "Users/Index".into(), hashmap![
        "users" => InertiaProp::data(User::all().await),
    ])
}

#[post("/users/store")]
async fn store(req: HttpRequest, body: Json<CreatUserDto>) -> impl Responder {
    let data = match body.validate_or_back(&req) {
        Err(err_redirect) => return err_redirect,
        Ok(data) => data,
    };

    if let Err(err) => User::create(data).await {
        return Inertia::back_with_errors(&req, hashmap![
            "error" => "Oops, we could not create this user. Try again later."
        ]);
    };

    Redirect::to("/users").see_other()
}

// the validation struct
#[derive(Deserialize, Validator)]
struct CreateUserDto {
    #[validate(
        required(message = "First name is a mandatory field."),
        length(max = 50, message = "Your first name must be shorter than 50 characters".)
    )]
    first_name: Option<String>,

    #[validate(
        required(message = "Last name is a mandatory field."),
        length(max = 50, message = "Your last name must be shorter than 50 characters".)
    )]
    last_name: Option<String>,

    #[validate(
        required(message = "Email is a mandatory field."),
        length(max = 50, message = "Your e-mail address must be shorter than 50 characters".),
        email(message = "You must provide a valid e-mail address.")
    )]
    email: Option<String>
}
```
