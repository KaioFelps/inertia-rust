use inertia_rust::{InertiaError, TemplateResolverOutput, ViewData};
use std::path::Path;

pub const EXPECTED_RENDER: &str = r#"
<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
    <meta http-equiv="X-UA-Compatible" content="ie=edge">
    
</head>
<body>
    <div id="app" data-page={"component":"Index","props":{},"url":"/","version":"v1.0.0"}></div>
</body>
</html>
"#;

pub const EXPECTED_RENDER_W_PROPS: &str = "
<!doctype html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0\">
    <meta http-equiv=\"X-UA-Compatible\" content=\"ie=edge\">
    \t\t
</head>
<body>
    <div id=\"app\" data-page={\"component\":\"Index\",\"props\":{\"user\":\"John Doe\"},\"url\":\"/withprops\",\"version\":\"v1.0.0\"}></div>
</body>
</html>";

async fn _mocked_resolver(
    template_path: &str,
    view_data: ViewData,
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

pub fn mocked_resolver(
    template_path: &'static str,
    view_data: ViewData,
    _data: &(),
) -> TemplateResolverOutput {
    Box::pin(_mocked_resolver(template_path, view_data))
}
