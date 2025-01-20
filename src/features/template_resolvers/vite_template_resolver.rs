use crate::{template_resolver::TemplateResolver, InertiaError, ViewData};
use async_trait::async_trait;
use regex::Regex;
use serde_json::{to_value, Map, Value};
use std::{path::Path, sync::OnceLock};
use vite_rust::{features::html_directives::ViteDefaultDirectives, Vite};

static INERTIA_VIEW_DATA_REGEX: OnceLock<Regex> = OnceLock::new();

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

        view_data_directive(&mut html, &view_data.custom_props);

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

fn view_data_directive(html: &mut String, view_data: &Map<String, Value>) {
    // @inertia::view_date(isSsr)
    let regex =
        INERTIA_VIEW_DATA_REGEX.get_or_init(|| Regex::new(r"@inertia::view_data\((.*)\)").unwrap());

    let js_view_data = to_value(view_data).unwrap_or_default();

    *html = regex
        .replace_all(html, |caps: &regex::Captures| {
            let search = caps[1].to_string();
            let keys = search.split(".").collect::<Vec<_>>();

            let mut value = &js_view_data;
            let mut final_value = String::new();

            for key in keys {
                let v = value.get(key).unwrap_or(&Value::Null);
                if matches!(v, Value::Object(_)) {
                    value = v;
                    continue;
                };

                final_value = match v {
                    Value::String(v) => v.to_owned(),
                    _ => v.to_string(),
                }
            }

            final_value
        })
        .to_string();
}

#[cfg(test)]
mod test {
    use serde_json::{json, Map, Value};

    use crate::hashmap;

    use super::view_data_directive;

    #[test]
    fn view_data_directive_can_access_direct_keys() {
        let custom_view_data = Map::from_iter(hashmap![
            "title".into() => Value::String("My awesome custom title".into()),
            "isSsr".into() => Value::Bool(true),
            "body".into() => Value::String("<h1>Hello World!</h1>".into())
        ]);

        let mut html = r#"
            <!doctype html>
            <html lang="pt-BR" class="h-full">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
                <meta http-equiv="X-UA-Compatible" content="ie=edge">
                <title inertia>@inertia::view_data(title)</title>
                <meta name="ssr" content="@inertia::view_data(isSsr)">
            </head>
            <body class="h-full bg-purple-100">
                @inertia::view_data(body)
            </body>
            </html>
        "#.to_string();

        view_data_directive(&mut html, &custom_view_data);

        let expected = r#"
            <!doctype html>
            <html lang="pt-BR" class="h-full">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
                <meta http-equiv="X-UA-Compatible" content="ie=edge">
                <title inertia>My awesome custom title</title>
                <meta name="ssr" content="true">
            </head>
            <body class="h-full bg-purple-100">
                <h1>Hello World!</h1>
            </body>
            </html>
        "#;

        assert_eq!(html, expected);
    }

    #[test]
    fn view_data_directive_can_access_nested_keys() {
        let custom_view_data = Map::from_iter(hashmap![
            "isSsr".into() => Value::Bool(true),
            "body".into() => json!({
                "h1": "<h1>Hello World!</h1>",
                "main": "<main>Body is a nested value!</main>",
                "meta": {
                    "title": "My awesome custom title"
                }
            })
        ]);

        let mut html = r#"
            <!doctype html>
            <html lang="pt-BR" class="h-full">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
                <meta http-equiv="X-UA-Compatible" content="ie=edge">
                <title inertia>@inertia::view_data(body.meta.title)</title>
                <meta name="ssr" content="@inertia::view_data(isSsr)">
            </head>
            <body class="h-full bg-purple-100">
                @inertia::view_data(body.h1)
                @inertia::view_data(body.main)
            </body>
            </html>
        "#.to_string();

        view_data_directive(&mut html, &custom_view_data);

        let expected = r#"
            <!doctype html>
            <html lang="pt-BR" class="h-full">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
                <meta http-equiv="X-UA-Compatible" content="ie=edge">
                <title inertia>My awesome custom title</title>
                <meta name="ssr" content="true">
            </head>
            <body class="h-full bg-purple-100">
                <h1>Hello World!</h1>
                <main>Body is a nested value!</main>
            </body>
            </html>
        "#;

        assert_eq!(html, expected);
    }
}
