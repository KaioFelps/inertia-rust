# Changelog

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

### Template Resolvers
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

### `reflash_inertia_session` method
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