use crate::{Component, InertiaError, InertiaProps};
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait InertiaFacade<TRequest, TResponse> {
    /// Renders an Inertia Page as an HTTP response.
    ///
    /// # Arguments
    /// * `req`         -   A reference to the HTTP request.
    /// * `component`   -   The page javascript component name to be rendered by the
    ///                     client-side adapter.
    ///
    /// # Panic
    /// Panics if Inertia instance hasn't been configured (set to AppData).
    async fn render(req: &TRequest, component: Component) -> Result<TResponse, InertiaError>;

    /// Renders an Inertia Page with props as an HTTP response.
    ///
    /// # Arguments
    /// * `req`         -   A reference to the HTTP request.
    /// * `component`   -   The page component to be rendered by the client-side adapter.
    /// * `props`       -   A `TProps` (serializable) struct containing
    ///                     the props to be sent to the client-side.
    ///
    /// # Errors
    /// This operation may result in one of InertiaErrors if the props struct
    /// or any of its fields don't implement [`Serialize`] trait.
    ///
    /// # Panics
    /// Panics if Inertia instance hasn't been configured (set to AppData).
    ///
    /// [`Serialize`]: serde::Serialize
    async fn render_with_props(
        req: &TRequest,
        component: Component,
        props: InertiaProps<'_>,
    ) -> Result<TResponse, InertiaError>;

    /// Provokes a client-side redirect to an extern URL.
    ///
    /// # Arguments
    /// * `req`     - A reference to the HTTP request.
    /// * `url`     - The URL to be redirected to.
    fn location(req: &TRequest, url: &str) -> TResponse;

    /// Whether to encrypt or not the current request. Refer to [History Encrypt] for more
    /// details
    ///
    /// [History Encrypt]: https://kaiofelps.github.io/inertia-rust/history-encrypt
    fn encrypt_history(req: &TRequest, encrypt: bool);
}
