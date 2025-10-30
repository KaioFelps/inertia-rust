# Changelog

## Unreleased
### Changed
- Undeserializable responses from SSR server now are parsed as string and added to the error message.

## v2.4.4
### Changed
- Logs Inertia page in debug on every render & when a rendering fails (even if logs are not in debug mode).

### Removed
- `http_method` module with `HttpMethod` enum — this is not considered a breaking change since the enum
    were not used within the application nor relevant enough to be using outside of it.

## v2.4.3
### Changed
- `InertiaTestRequest` implementation for actix-web now checks for request's Inertia header over response's.

## v2.4.2
### Added
- `AssertableInertia` and methods for extracting it from test request/responses;
- Testing guide in the docs.

## v2.4.1
### Changed
- `Inertia::back` now returns to url suggested by `referer` header rather than session's previous url.

## v2.4.0
### Added
- `ViteHBSTemplateResolver`, which uses Handlebars as template engine.

### Changed
- Merged `inner_render` and `inner_render_with_props` methods;
- Updated the docs for using `ViteHBSTemplateResolver` by default;
- Deprecated `ViteTemplateResolver`.

### Removed
- `template_path` field and setters from `Inertia` and `InertiaConfigBuilder`;

### Breaking
#### Template Resolvers
`template_path` has been removed from Inertia struct. Now, it's the template resolver's responsability to fetch and
parse the template HTML. This means `ViteTemplateResolver` now is the one who takes the template path on initialization.

Also, its `new` method now returns a `Result<ViteTemplateResolver, InertiaError>` instead of `ViteTemplateResolver`
directly, since it looks for the root template in the given path and return an error if it can't find the file.

```diff
let vite = initialize_vite().await;
- let resolver = ViteTemplateResolver::new(vite);
+ let resolver = ViteTemplateResolver::new(vite, "www/root.html").unwrap();

let inertia = Inertia::new(
    InertiaConfig::builder()
        .set_url("http://localhost:8080")
        .set_version(InertiaVersion::Literal(ASSETS_VERSION.get().unwrap()))
-       .set_template_path("www/root.html")
        .set_template_resolver(Box::new(resolver))
        .enable_ssr()
        .set_ssr_client(SsrClient::new("127.0.0.1", 1000))
        .build(),
);
```

## v2.3.8
### Fixed
- Add `Vary: X-Inertia` header to Inertia responses. This fixes some cache-related issues (for instance, a plain JSON
    response when navigating through browser history, instead of the actual HTML rendered page).

## v2.3.7
### Fixed
- Also include deferred and mergeable props from shared props (Inertia Middleware) in the inertia page response.

## v2.3.6
### Added
- Added new features: `validator`, `actix-validator`;
- Added `InertiaValidateOrRedirect` trait for validating or generating an error redirect
    (if `validator` feature is enabled);
- Implemented `InertiaValidateOrRedirect` for actix-web (if `actix-validator` is enabled).

### Changed
- `Inertia::back_with_errors` hashmap keys are now anything that implements `ToString` trait.

## v2.3.5
### Fixed
- `is_inertia_response` method.

## v2.3.4
### Added
- `@inertia::view_data()` directive for `ViteTemplateResolver`;
- `Inertia::view_data` facade method;
- New view data methods to documentation.

### Removed
- Inertia global `view_data` struct and related options -- from `InertiaConfigBuilder`.

## v2.3.3
### Added
- `is_inertia_response` method.

### Changed
- `ReflashTemporarySessionMiddleware` implementation in the documentation.

## v2.3.2
### Fixed
- Partial requests would receive props that haven't been requested if `only` header is empty;
- `InertiaProp::resolve_props` would always evaluate lazy props.

## v2.3.1
### Changed
- `IntoInertiaError` now is public.

## v2.3.0
### Added
- `IntoInertiaPropResult` trait, which introduces `into_inertia_value`.

### Changed
- `InertiaProp` enums now require either `Result<Value, InertiaError>` or an async callback that returns it;
- `InertiaProp` enum constructors automatic serializes and map the error to `InertiaError`;
- `render_with_props` now will immediately return an `InertiaError` if any prop fails to be resolved.

### Breaking Changes
#### Serializing Props
Instead of calling `to_value(...).unwrap()`, you must call `into_inertia_value` into a serializable object.
This applies for `InertiaProp`s which takes a callback and also for any prop that is being instantiating
directly.

```diff
use inertia_rust::{
    hashmap,
    prop_resolver,
    InertiaProp,
+   IntoInertiaError;
};
- use serde_json::to_value;

hashmap![
-   "foo" => InertiaProp::Data(to_value("Foo").unwrap()),
+   "foo" => InertiaProp::Data("Foo".into_inertia_value()),
    "users" => InertiaProp::defer(prop_resolver!(
            let users_clone = users.clone(); {
            let counter = TIMES_DEFERRED_RESOLVER_HAS_EXECUTED.get_or_init(|| Arc::new(Mutex::new(0)));
            *counter.lock().unwrap() += 1;

-           to_value(users_clone
+           users_clone
                .clone()
                .iter()
                .skip((page -1)* per_page)
                .take(per_page)
                .cloned()
                .collect::<Vec<_>>()
-           ).unwrap()
+               .into_inertia_value()
        }))
        .into_mergeable(),
        "permissions" => InertiaProp::merge(permissions.into_iter().skip((page-1)*per_page).take(per_page).collect::<Vec<_>>())
],
```

## v2.2.0
### Changed
- Inertia Middleware `with_shared_props` return type is now a async callback, so that props can be
    asynchronously resolved from inside the middleware.

### Breaking Changes
#### Inertia Middleware
When sharing props, instead of:
```rust
InertiaMiddleware::new().with_shared_props(Arc::new(|_req: &ServiceRequest| {
    hashmap![ "foo" => InertiaProp::Always("bar".into()) ]
})),
```

do:
```rust
InertiaMiddleware::new().with_shared_props(Arc::new(move |_req: &HttpRequest| {    
    async move {
        hashmap![ "foo" => InertiaProp::Always("bar".into()) ]
    }
    .boxed_local()
})),
```

## v2.1.0
### Changed
- lowered min required version for `tokio` and `actix-web` crates;
- fixed `inertia_rust` version in the installation chapter, at the `Cargo.toml` snippet.

## v2.0.0
> [!WARNING]
> We've jumped straight to v2.0 in order to keep up with Inertia.js versions, as this is the respective inertia-rust
> version for dealing with Inertia.js 2.

### Added
- support for [Deferred Props](https://inertiajs.com/deferred-props);
- support for [Merge Props](https://inertiajs.com/merging-props);
- `InertiaFacade` trait + implementation for actix-web provider;
- `Inertia::back` and `Inertia::back_with_errors` methods;
- documentation website.

### Removed
- `template_resolver_data` field from `InertiaConfig` and `InertiaConfigBuilder`;
- `inertia_rust::actix::render` and `inertia_rust::actix::render_with_props` facade methods;
- `reflash_inertia_session` usage from crate (and setters from `InertiaConfigBuilder`);

### Changed
- `Inertia::template_resolver` field's type;

### Breaking Changes
As the breaking changes that has occurred are important and too complex for a simple bullet list, they'll be better
described in their own sub-topics below.

#### Template Resolvers
Before, you'd specifya reference to a closure returning a boxed async function that would actually resolve your
template. Also, it'd require another field specifying the third parameter to be passed to the closure (by the
Inertia rendering methods). What if you don't actually need this third parameter? You would need to pass an `&()`
to the `template_resolver_data` field!

Now, the data field no longer exists. The `template_resolver`, on the other hand, require a `Box`ed struct that
implements our `TemplateResolver` trait. It's absolutely more simple now. You can place any struct your resolver
might need directly in the struct body. Your implementation now can also be a simple
`async fn foo(...) { /* ... */ }`, instead of something like `let foo = move |...| Box::pin(async move { /* ... */ })`.

This is better explained on the **[Template Resolvers]** section from the documentation.

[Template Resolvers]: https://kaiofelps.github.io/inertia-rust/basic/advanced/template_resolvers.html

#### `reflash_inertia_session` method
This field was optional and would default to a useless callback (something like `|_| Ok(())`). It'd be used for
reflashing the session when Inertia Rust would decide to trigger a forced-refresh --- due to assets version
mismatch.

Now, instead of calling it, Inertia Rust will add the `InertiaTemporarySession` from struct (if there is such) to
the request extensions wrapped with `InertiaSessionToReflash`. It's still up to you to guarantee it's reflashed using
sessions or whatever method you decides to.

#### Actix Facades
`actix::facade`'s `render` and `render_with_props` methods no longer exist. Now, you might use `Inertia::render`
and `Inertia::render_with_props` methods as the actix-web facade replacements.

```rust
// before
use vite_rust::Vite;
use inertia_rust::actix::render_with_props;
render_with_props::<Vite>(&req, "Index".into(), props).await

// now
use inertia_rust::{Inertia, InertiaFacade};
Inertia::render_with_props(&req, "Index".into(), props).await
```

#### Inertia `render` and `render_with_props` methods
Previously, you'd need to specify the type of the `template_resolver_data` on every render method (even with the
facade's ones). It's not needed anymore, since `template_resolver_data` doesn't even exist now!

```rust
// considering you're using vite-rust
// before
let inertia: inertia_rust::Inertia;
inertia.render::<vite_rust::Vite>(...);

// after
let inertia: inertia_rust::Inertia;
inertia.render(...);
```

But, this is neither how to do it correctly now. Indeed, `render` and `render_with_props` has been renamed to,
respectively, `inner_render` and `inner_render_with_props`. The reason is you're not supposed to use it directly.
Instead, we encourage you to use your provider's facades:

```rust
use inertia_rust::Inertia;
// DON'T    ❌
let inertia: Inertia;
inertia.inner_render(...);

// DO       ✅
Inertia::render(...);
```

Internally, the facade will extract the configured `Inertia` instance from the request. You can notice that you
need to pass the Http Request as paramter to the facade method, but you'd also need to do it for the instance
inner method, so there is literally 0 advantage on using the instance methods directly.

---

## v0.1.0
### Added
- `InertiaProp`s;
- render methods (with or without props);
- acitx-web provider;
- render facades for actix-web provider;
- `InertiaConfig` and `InertiaConfigBuilder` for instantiating `Inertia` struct;
- `NodeJsProc` and server-side rendering feature;
- vite-rust template resolver;
- `InertiaMiddleware` with shared props.