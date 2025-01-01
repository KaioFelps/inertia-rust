use crate::{template_resolver::TemplateResolver, InertiaError, ViewData};
use async_trait::async_trait;
use std::path::Path;
use vite_rust::{features::html_directives::ViteDefaultDirectives, Vite};

pub struct BasicViteResolver {
    vite: Vite,
}

impl BasicViteResolver {
    pub fn new(vite: Vite) -> Self {
        Self { vite }
    }
}

#[async_trait(?Send)]
impl TemplateResolver for BasicViteResolver {
    async fn resolve_template(
        &self,
        template_path: &str,
        view_data: ViewData<'_>,
    ) -> Result<String, InertiaError> {
        let path = Path::new(template_path);
        let file = match tokio::fs::read(&path).await {
            Ok(file) => file,
            Err(err) => {
                return Err(InertiaError::RenderError(format!(
                    "Failed to open root layout at {}: {:#}",
                    path.to_str().unwrap(),
                    err
                )))
            }
        };

        let mut html = match String::from_utf8(file) {
            Err(err) => {
                return Err(InertiaError::RenderError(format!(
                    "Failed to read file contents: {err:?}"
                )))
            }
            Ok(html) => html,
        };

        if let Err(err) = self.vite.vite_directive(&mut html) {
            log::warn!("Failed to resolve vite directive: {}", err);
        };

        self.vite.assets_url_directive(&mut html);
        self.vite.hmr_directive(&mut html);
        self.vite.react_directive(&mut html);

        match &view_data.ssr_page {
            Some(ssr) => {
                html = html.replace("@inertia::body", ssr.get_body());
                html = html.replace("@inertia::head", &ssr.get_head());
            }
            None => {
                let stringified_page: Result<String, serde_json::Error> =
                    serde_json::to_string(&view_data.page);

                if stringified_page.is_err() {
                    return Err(InertiaError::SerializationError(format!(
                        "Failed to serialize view_data.page: {:?}",
                        &view_data.page
                    )));
                }

                let stringified_page = stringified_page.unwrap();
                let container = format!("<div id='app' data-page='{}'></div>\n", stringified_page,);

                html = html.replace("@inertia::body", &container);
                html = html.replace("@inertia::head", "");
            }
        }

        Ok(html)
    }
}
