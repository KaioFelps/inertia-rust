use crate::{InertiaError, TemplateResolverOutput, ViewData};
use std::path::Path;
use vite_rust::{features::html_directives::ViteDefaultDirectives, Vite};

// The async resolver
pub fn template_resolver(
    template_path: &'static str,
    view_data: ViewData,
    vite: &'static Vite,
) -> TemplateResolverOutput {
    Box::pin(async move {
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

        vite.vite_directive(&mut html);
        vite.assets_url_directive(&mut html);
        vite.hmr_directive(&mut html);
        vite.react_directive(&mut html);

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
    })
}
