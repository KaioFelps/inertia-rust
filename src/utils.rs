use std::time::Duration;

use crate::error::InertiaError;
use crate::{InertiaPage, InertiaSSRPage};

pub(crate) async fn request_page_render(
    server_url: &reqwest::Url,
    page: &InertiaPage<'_>,
) -> Result<InertiaSSRPage, InertiaError> {
    let mut render_endpoint = server_url.clone();
    render_endpoint.set_path("render");

    let response = reqwest::Client::new()
        .get(render_endpoint)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&page)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|err| {
            InertiaError::SsrError(format!(
                "Failed to render the page at the SSR Server: {}",
                err
            ))
        })?;

    let body = response
        .bytes()
        .await
        .map_err(|err| {
            InertiaError::SsrError(format!("Failed to read SSR response's bytes: {err}"))
        })?
        .to_vec();

    serde_json::from_slice::<InertiaSSRPage>(&body).map_err(|json_err| {
        let body_as_text = String::from_utf8(body);
        match body_as_text {
            Ok(text_body) => InertiaError::SsrError(format!(
                "Failed to deserialize InertiaSSRPage object \
                from body with error: {json_err}. Received body: {text_body}"
            )),
            Err(text_error) => InertiaError::SsrError(format!(
                "Failed to deserialize InertiaSSRPage object \
                from body with error: {json_err}. Also failed to read bytes as text \
                with error: {text_error}",
            )),
        }
    })
}
