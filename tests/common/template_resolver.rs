use async_trait::async_trait;
use inertia_rust::{template_resolvers::TemplateResolver, InertiaError, ViewData};
use std::path::Path;

use crate::super_trim;

pub fn get_dynamic_csr_expect(url: &str, props: &str, component: &str, version: &str) -> String {
    super_trim(format!(
        r#"
<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
    <meta http-equiv="X-UA-Compatible" content="ie=edge">
    
</head>
<body>
    <div id="app" data-page={{"component":"{}","props":{},"url":"{}","version":"{}","clearHistory":false,"encryptHistory":false}}></div>
</body>
</html>"#,
        component, props, url, version
    ))
}

pub struct MockedTemplateResolver;

#[async_trait(?Send)]
impl TemplateResolver for MockedTemplateResolver {
    async fn resolve_template(
        &self,
        template_path: &str,
        view_data: ViewData<'_>,
    ) -> Result<String, InertiaError> {
        let path = Path::new(template_path);

        let read_file = tokio::fs::read(&path).await;

        if read_file.is_err() {
            return Err(InertiaError::SsrError(format!(
                "Failed to open root layout at {}: {:#}",
                path.to_str().unwrap(),
                read_file.unwrap_err()
            )));
        }

        let data = read_file.unwrap();

        let mut html = match String::from_utf8(data) {
            Err(err) => {
                return Err(InertiaError::SsrError(format!(
                    "Failed to read file contents: {err:?}"
                )))
            }
            Ok(html) => html,
        };

        match view_data.ssr_page {
            Some(ssr) => {
                html = html.replace("%-inertia_body-%", ssr.get_body());
                html = html.replace("%-inertia_head-%", &ssr.get_head());
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

                let container = format!("<div id=\"app\" data-page={stringified_page}></div>",);
                html = html.replace("%-inertia_body-%", &container);
                html = html.replace("%-inertia_head-%", "");
            }
        }

        Ok(html)
    }
}
