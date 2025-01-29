# Template Resolvers

Template resolvers are structs that implements Inertia Rust's `TemplateResolver` trait. They're responsible
for rendering the root template from your application. Specially, for injecting Inertia.js head and body
into the HTML.

In summary, a template resolver has the following responsabilities:
- Inject assets tags (e.g. script, css and link tags), usually via some bundler;
- Inject Inertia SSRrendered head (or nothing if CSRendered);
- Inject Inertia body (SSR or CSRendered);
- Inject custom view data (if any).

For instance, the default `ViteTemplateResolver` (available when `vite-template-resolver` feature is enabled)
will use Vite Rust crate to inject the correct HTML tags for the application assets in the HTML (e.g.: React.js
scripts, the application entrypoint script, stylesheets links, preload tags, etc). For handling the other
responsabilities, it uses simple REGEXes to parse the available directives.

## Creating a Template Resolver

You can create your own template resolver pretty easily. All you need to do is create a struct with whatever
data your template resolver needs to render a page. Then, implement `TemplateResolver` trait for it.

For example, this is a short of `ViteTemplateResolver`:

```rust
use crate::{template_resolvers::TemplateResolver, InertiaError, ViewData};
use async_trait::async_trait;
use std::path::Path;
use vite_rust::{features::html_directives::ViteDefaultDirectives, Vite};

// It can contain anything you might need to resolve the template
pub struct ViteTemplateResolver {
    pub vite: Vite,
    template_root: String
}

impl ViteTemplateResolver {
    pub fn new(vite: Vite, template_path: &str) -> Self {
        let file = std::fs::read(template_path).expect("file {} should exist.", template_path);
        let template_root = String::from_utf8(file).expect("template file should contain a valid utf-8 encoded text.");

        Ok(Self { vite, template_root })
    }
}

#[async_trait(?Send)]
impl TemplateResolver for ViteTemplateResolver {
    async fn resolve_template(
        &self,
        view_data: ViewData<'_>,
    ) -> Result<String, InertiaError> {
        let mut html = self.template_root.clone();

        let _ self.vite.vite_directive(&mut html);
        self.vite.assets_url_directive(&mut html);
        self.vite.hmr_directive(&mut html);
        self.vite.react_directive(&mut html);

        if let Some(ssr) =  &view_data.ssr_page {
            html = html.replace("@inertia::body", ssr.get_body());
            html = html.replace("@inertia::head", &ssr.get_head());
        } else {
            let stringified_page = serde_json::to_string(&view_data.page).unwrap();
            let container = format!("<div id='app' data-page='{}'></div>\n", stringified_page);

            html = html.replace("@inertia::body", &container);
            html = html.replace("@inertia::head", "");
        }

        Ok(html)
    }
}
```

Naturally, there is no untreated `.unwrap()` or `.expect()` in the built-in template resolvers, and every error is
properly handled.

## The Vite and Handlebars Template Resolver

This template resolver is enabled through the `vite-hbs-template-resolver` feature. It uses [Handlebars], one of the most reliable and production-ready template engines available for Rust.

Through this template resolver, we expose a bunch of properties to you:

- `inertia_head`: the head of your Inertia page component.
- `inertia_body`: the body of your Inertia page component.
- `page`: a HashMap containing all of your Inertia page component props (yes, you can directly access them from your
    template like `page.user.name`, if it's a valid prop sent from your back-end).
- `view_data`: a HashMap containing the view data custom props. You provide it by using `Inertia::view_data()` method.
    Check [the Response guide] for more details.
- `vite`: the vite scripts (already resolved according to the environment [development or production]).
- `vite_react_refresh`: the script for refreshing your React application on development.

[Handlebars]: https://crates.io/crates/handlebars
[the Response guide]: ../basic/responses.md#root-template-data
