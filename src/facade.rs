use std::collections::HashMap;

use crate::{Component, InertiaError, InertiaProps};
use async_trait::async_trait;
use serde_json::Value;

#[async_trait(?Send)]
pub trait InertiaFacade<THttpRequest, TResponse, TRedirect> {
    /// Renders an Inertia Page as an HTTP response.
    ///
    /// # Arguments
    /// * `req`         -   A reference to the HTTP request.
    /// * `component`   -   The page javascript component name to be rendered by the
    ///                     client-side adapter.
    ///
    /// # Panic
    /// Panics if Inertia instance hasn't been configured (set to AppData).
    async fn render(req: &THttpRequest, component: Component) -> Result<TResponse, InertiaError>;

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
        req: &THttpRequest,
        component: Component,
        props: InertiaProps<'_>,
    ) -> Result<TResponse, InertiaError>;

    /// Provokes a client-side redirect to an extern URL.
    ///
    /// # Arguments
    /// * `req`     - A reference to the HTTP request.
    /// * `url`     - The URL to be redirected to.
    fn location(req: &THttpRequest, url: &str) -> TResponse;

    /// Whether to encrypt or not the current request. Refer to [History Encrypt] for more
    /// details.
    ///
    /// [History Encrypt]: https://kaiofelps.github.io/inertia-rust/history-encrypt
    fn encrypt_history(req: &THttpRequest, encrypt: bool);

    /// Triggers a history clearing from server-side. Refer to [History Encrypt] for more
    /// details.
    ///
    /// [History Encrypt]: https://kaiofelps.github.io/inertia-rust/history-encrypt#clearing-history
    fn clear_history(req: &THttpRequest);

    /// Triggers a redirect to the previous URL (or "/", if there is no previous URL).
    /// Please refer to [Flash Messages and Validation Errors] for more information.
    ///
    /// [Flash Messages and Validation Errors]: https://kaiofelps.github.io/inertia-rust/advanced/flash-messages.html
    fn back(req: &THttpRequest) -> TRedirect;

    /// Triggers a redirect to the previous URL (or "/", if there is no previous URL) with
    /// errors. Please refer to [Flash Messages and Validation Errors] for more information.
    ///
    /// [Flash Messages and Validation Errors]: https://kaiofelps.github.io/inertia-rust/advanced/flash-messages.html
    fn back_with_errors<Key: ToString>(
        req: &THttpRequest,
        errors: HashMap<Key, Value>,
    ) -> TRedirect;

    /// Inserts custom view data to be accessed by the template root.
    fn view_data(req: &THttpRequest, data: HashMap<&str, Value>);

    /// Checks whether the request was made by the Inertia client router rather than a standard full-page load.
    fn check_is_inertia_request(req: &THttpRequest) -> bool;
}
