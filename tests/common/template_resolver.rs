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
    <meta name="ssr" value="false">
    <title inertia></title>
    
</head>
<body>
    <div id="app" data-page={{"component":"{}","props":{},"url":"{}","version":"{}","clearHistory":false,"encryptHistory":false}}></div>
</body>
</html>"#,
        component, props, url, version
    ))
}

pub fn get_dynamic_csr_with_view_data_expect(
    url: &str,
    props: &str,
    component: &str,
    version: &str,
    title: &str,
    is_ssr: bool,
) -> String {
    super_trim(format!(
        r#"
<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
    <meta http-equiv="X-UA-Compatible" content="ie=edge">
    <meta name="ssr" value="{}">
    <title inertia>{}</title>
    
</head>
<body>
    <div id="app" data-page={{"component":"{}","props":{},"url":"{}","version":"{}","clearHistory":false,"encryptHistory":false}}></div>
</body>
</html>"#,
        is_ssr, title, component, props, url, version
    ))
}

pub struct MockedTemplateResolver {
    root_template: String,
}

impl MockedTemplateResolver {
    pub fn new(template_path: &str) -> Result<Self, InertiaError> {
        let path = Path::new(template_path);

        let file_data = std::fs::read(path).map_err(|err| {
            InertiaError::RenderError(format!(
                "Failed to open root layout at {}: {:#}",
                path.to_str().unwrap(),
                err
            ))
        })?;

        let root_template = String::from_utf8(file_data).map_err(|err| {
            InertiaError::RenderError(format!("Failed to read file contents: {err:?}"))
        })?;

        Ok(Self { root_template })
    }
}

#[async_trait(?Send)]
impl TemplateResolver for MockedTemplateResolver {
    async fn resolve_template(&self, view_data: ViewData<'_>) -> Result<String, InertiaError> {
        let mut html = self.root_template.clone();

        html = html.replace(
            "%-view_data_title-%",
            view_data
                .custom_props
                .get("title")
                .map(|v| v.as_str().unwrap())
                .unwrap_or(""),
        );

        html = html.replace(
            "%-view_data_ssr-%",
            &view_data
                .custom_props
                .get("isSsr")
                .map(|v| v.as_bool().unwrap())
                .unwrap_or_default()
                .to_string(),
        );

        match view_data.ssr_page {
            Some(ssr) => {
                html = html.replace("%-inertia_body-%", &ssr.get_body());
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
