# Template Resolvers

Template resolvers are structs that implements Inertia Rust's `TemplateResolver` trait. They're responsible
for rendering the root template from your application. Specially, for injecting Inertia.js head and body
into the HTML.

For instance, the default `ViteTemplateResolver` (available when `vite-template-resolver` feature is enabled)
will use Vite Rust crate to inject the correct HTML tags for the application assets in the HTML (e.g.: React.js
scripts, the application entrypoint script, stylesheets links, preload tags, etc). Also, it'll handle server-
side rendering and correctly insert Inertia's body and head in the template (if it's SSRendered) or the
`InertiaPage` in the container element (if CSRendered).

## Creating a Template Resolver

You can create your own template resolver pretty easily. All you need to do is create a struct with whatever
data your template resolver needs to render a page. Then, implement `TemplateResolver` trait for it.

For instance, this is a short of `ViteTemplateResolver`:

```rust
use crate::{template_resolvers::TemplateResolver, InertiaError, ViewData};
use async_trait::async_trait;
use std::path::Path;
use vite_rust::{features::html_directives::ViteDefaultDirectives, Vite};

pub struct ViteTemplateResolver {
    pub vite: Vite,
}

impl ViteTemplateResolver {
    pub fn new(vite: Vite) -> Self {
        Self { vite }
    }
}

#[async_trait(?Send)]
impl TemplateResolver for ViteTemplateResolver {
    async fn resolve_template(
        &self,
        template_path: &str,
        view_data: ViewData<'_>,
    ) -> Result<String, InertiaError> {
        let path = Path::new(template_path);
        let file = match tokio::fs::read(&path).await.unwrap();

        let mut html = String::from_utf8(file).unwrap()

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

Naturally, there is no untreated `.unwrap()` in the official `ViteTemplateResolver`, and every error is
properly handled.
