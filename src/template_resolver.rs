use async_trait::async_trait;

use crate::{InertiaError, ViewData};

#[async_trait(?Send)]
pub trait TemplateResolver {
    /// A function responsible for rendering the root template
    /// with the given **view data**.
    ///
    /// This should be relative by the template engine you are using, and it is mandatory for
    /// rendering the HTML to be served on full requests. Since Rust does not offer a standard
    /// template engine, there are various options, and it is not our goal to tie you to a specific
    /// one which we opted to use.
    ///
    /// # Arguments
    /// Inertia will call this function passing the following parameters to it:
    /// * `view_data`   -   A [`ViewData`] struct,
    ///
    /// # Errors
    /// Returns an [`InertiaError::RenderError`] if it fails to render the html.
    ///
    /// # Return
    /// The return must be the template rendered to HTML. It will be sent as response to full
    /// requests.
    async fn resolve_template(&self, view_data: ViewData<'_>) -> Result<String, InertiaError>;
}
