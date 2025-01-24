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
        .await;

    let response = match response {
        Err(err) => {
            return Err(InertiaError::SsrError(format!(
                "Failed to render the page at the SSR Server: {}",
                err
            )))
        }
        Ok(response) => response,
    };

    match response.json::<InertiaSSRPage>().await {
        Err(err) => Err(InertiaError::SsrError(format!(
            "Failed to desserialize InertiaSSRPage object: {}",
            err
        ))),
        Ok(page) => Ok(page),
    }
}
