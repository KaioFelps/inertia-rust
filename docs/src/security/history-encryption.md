# History Encryption

History encryption is a way of protecting the page data stored in the browser history. For more details
about it, refer to the official Inertia.js [History encryption] documentation. In this page, you can see
how to enable it in Inertia Rust.

[History encryption]: https://inertiajs.com/history-encryption

## Global Encryption

On your Inertia struct config, you can `encrypt_history` to globally enable history encryption:

```rust
use inertia_rust::{Inertia, InertiaConfig};

let inertia = Inertia::new(
        InertiaConfig::builder()
            .encrypt_history()
            .build());
```

It'll set `Inertia::encrypt_history` to `true`. By default, it's disabled.

## Per-request Encryption

Call `encrypt_history(bool)` to enable or disable encryption of a request before you render the page:

```rust
Inertia::encrypt_history(&req, true);
```

It overwrites the global configuration and even the middleware (we'll cover it in the next topic). It
means you can use it to disable encryption for an specific route by passing `false` as parameter.

## Encrypt Middleware

You can import the `EncryptHistoryMiddleware` from your provider and wrap some routes with it to enable
encryption for the routes within the wrapped group.

```rust
use actix_web::App;
use inertia_rust::actix::EncryptHistoryMiddleware;

App::new().wrap(EncryptHistoryMiddleware::new());
```

## Clearing History

To clear the history state from the server-side, call `clear_history` before rendering the page:

```rust
Inertia::clear_history(&req):
```

Again, for more information about clearing history, refere to Inertia's [Clearing history] section from
History encryption documentation.

[Clearing history]: https://inertiajs.com/history-encryption#clearing-history
