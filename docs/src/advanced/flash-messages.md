# Flash Messages and Validation Errors

[Inertia Middleware](../basic/installation.md#inertia-middleware) will also merge flash errors with the
shared props. The resulting props will be injected back into the request and will be further merged again
with the page props during rendering, thus, making all of them available to your client-side page component.

As said earlier, Inertia Rust is not made for one single framework and all of them actually might have
built-in sessions management. Hence, you need to built by yourself a second middleware that injects an `InertiaTemporarySession` object in the request context/extensions:

```rust
// InertiaTemporarySession struct from Inertia Rust
#[derive(Clone, Serialize)]
pub struct InertiaTemporarySession {
    // Optional errors hashmap
    pub errors: Option<Map<String, Value>>,
    // The previous request URL
    // useful for redirecting back with errors
    pub prev_req_url: String,
}
```

The middleware tries to extract this from the request context and merge it with the shared props. This
is how validation errors might get available to your page components without explicitly sending them with `Inertia::render_with_props`.

It's up to you to set up a middleware that extracts errors from the session and add them wrapped in
`InertiaTemporarySession` to the request's extensions, however, there is a useful snippet containing a
sample of this middleware in [Actix Web Implementations] chapter.

## Reflash Session Middleware

Sometimes, Inertia Rust will trigger a forced-refresh (see [Assets Management] topic from Inertia.js
official documentation for more details). If there was a `InertiaTemporarySession` to be treated by
that request, it'll be lost by the nature of temporary sessions.

To avoid this, Inertia Rust wraps the current temporary session inside another struct ---
`InertiaSessionToReflash`. This allows you to make a middleware responsible for using it to reflash the
session. This way, the temporary session will be available for the subsequent request from the current
client.

Check a actix web implementation of this middleware at [Actix Web Implementations] chapter.

[Assets Management]: https://inertiajs.com/asset-versioning
[Actix Web Implementations]: ./actix-web-implementations.md#temporary-session-middleware-with-reflash

## Redirecting Back With Errors

To redirect back with errors, ensure the Inertia Temporary Session middleware is correctly set up. It
must look for a `SessionErrors` instance in the request extensions and persist it in the user's session
inside the `"_errors"` key -- or whatever key choosen to represent the errors in the sessions.

> Note: `"_errors"` key is expected to represent a stringified JSON object.

For instance, it would be something like the following pseudo Rust code:

```rust
use actix_web::{get, HttpRequest, Responder};
use inertia_rust::{hashmap, Inertia, InertiaFacade};

#[get("/foo")]
async fn foo(req: HttpRequest) -> impl Responder {
    Inertia::back_with_errors(&req, hashmap![
        "age" => to_value("You must be over 13 y.o. to access this website").unwrap()
    ])
}
```

For making it a little bit better, you might provide your own facade for redirecting back with errors.
Again, it's not possible for us to make this little helper, since it'd be necessary to introduce session
management crates as dependency. Nor can we provide the utility trait so that you implement it yourself,
since you can only implement your own traits for foreign struct.

However, you can refer to [Actix Web Implementations](./actix-web-implementations.md#the-back-with-errors-method)
and grab a snippet containing an sample implementation of a trait that enhances `Inertia` struct, providing
a `back_with_errors` method.


