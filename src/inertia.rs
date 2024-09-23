use std::future::Future;
use std::pin::Pin;
use async_trait::async_trait;
use serde::Serialize;
use serde_json::{Map, Value};
use crate::{InertiaError, InertiaSSRPage};
use crate::utils::{inertia_err_msg, inertia_panic};
use crate::props::InertiaProps;
use crate::req_type::InertiaRequestType;

pub const X_INERTIA: &'static str = "x-inertia";
pub const X_INERTIA_LOCATION: &'static str = "x-inertia-location";
pub const X_INERTIA_VERSION: &'static str = "x-inertia-version";
pub const X_INERTIA_PARTIAL_COMPONENT: &'static str = "x-inertia-partial-component";
pub const X_INERTIA_PARTIAL_DATA: &'static str = "x-inertia-partial-data";
pub const X_INERTIA_PARTIAL_EXCEPT: &'static str = "x-inertia-partial-except";

/// The javascript component name.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Component(pub String);

/// InertiaResponder trait defines methods that every crate feature
/// should implement. For instance, T may be a sort of actix-web Responder,
/// if "actix" feature is passed with the --feature flag or with the
/// feature field in the cargo toml.
#[async_trait(?Send)] // it's `?Send` because some frameworks like Actix won't require requests to be thread-safe
pub trait InertiaResponder<T, THttpReq> {
    /// Renders an Inertia Page as an HTTP response.
    ///
    /// # Arguments
    /// * `req`         -   The HTTP request.
    /// * `component`   -   The page javascript component name to be rendered by the
    ///                     client-side adapter.
    async fn render(&self, req: &THttpReq, component: Component) -> Result<T, InertiaError>;

    /// Renders an Inertia Page with props as an HTTP response.
    ///
    /// # Arguments
    /// * `req`         -   The HTTP request.
    /// * `component`   -   The page component to be rendered by the client-side adapter.
    /// * `props`       -   A `TProps` (serializable) struct containing
    ///                     the props to be sent to the client-side.
    ///
    /// # Errors
    /// This operation may result in one of InertiaErrors if the props struct
    /// or any of its fields don't implement [`Serialize`] trait.
    ///
    /// [`Serialize`]: serde::Serialize
    async fn render_with_props(&self, req: &THttpReq, component: Component, props: InertiaProps) -> Result<T, InertiaError>;

    fn redirect(&self, location: String) -> T;
}

/// Defines some helper methods to be implemented to HttpRequests from the
/// library opted by the cargo feature.
pub(crate) trait InertiaHttpRequest {
    fn is_inertia_request(&self) -> bool;

    fn get_request_type(&self) -> Result<InertiaRequestType, InertiaError>;
}

pub enum InertiaVersion {
    Literal(String),
    Resolver(fn() -> String)
}

/// View Data is a struct containing props to be used by the root template.
pub struct ViewData {
    pub page_props: Map<String, Value>,
    pub ssr_props: Option<InertiaSSRPage>,
    pub custom_props: Map<String, Value>
}

pub(crate) type TemplateResolver = fn(&'_ str, ViewData) -> Pin<Box<dyn Future<Output = Result<String, InertiaError>> + Send + '_>>;

pub struct SsrClient<T> {
    pub sender: hyper::client::conn::http1::SendRequest<T>
}

/// Inertia struct must be a singleton and initialized at the application bootstrap.
/// It is supposed to last during the whole application runtime.
///
/// Extra details of how to initialize and keep it is specific to the feature-opted http library.
pub struct Inertia {
    /// URL used between redirects and responses generation, i.g. "https://myapp.com".
    pub(crate) url: &'static str,
    /// The path to find the root html template to render everything in.
    pub(crate) template_path: &'static str,
    /// The current assets version.
    pub(crate) version: &'static str,
    /// A function responsible for rendering the root template
    /// with the given **view data** and/or **page data**.
    ///
    /// This should be relative by the template engine you are using, and it is mandatory for
    /// rendering the HTML to be served on full requests. Since Rust does not offer a standard
    /// template engine, there are various options, and it is not our goal to tie you to a specific
    /// one which we opted to use.
    ///
    /// # Arguments
    /// Inertia will call this function passing the following parameters to it:
    /// * `path`        -   The path to the application template (`Inertia::template_path`).
    /// * `view_data`   -   A [`ViewData`] struct,
    ///
    /// # Errors
    /// Returns an [`InertiaError::SsrError`] if it fails to render the html.
    ///
    /// # Returns
    /// The return must be the template rendered to HTML. It will be sent as response to full
    /// requests.
    pub(crate) template_resolver: TemplateResolver,
    /// A client to make request to Inertia Server (and render the page).
    pub(crate) ssr_client: Option<SsrClient<String>>,
    /// Extra data to be passed to the root template.
    pub(crate) custom_view_data: Map<String, Value>
}

impl Inertia {
    /// Initializes an instance of [`Inertia`] struct.
    ///
    /// # Arguments
    /// * `url`                 -   A valid [href] of the current application
    /// * `version`             -   The current asset version of the application.
    ///                             See [Asset versioning] for more details.
    /// * `template_path`       -   The path for the root html template.
    /// * `template_resolver`   -   A function that renders the given root template html. Check
    ///                             more details at [`Inertia::template_resolver`] doc string.
    ///
    /// [`Inertia`]: Inertia
    pub fn new(
        url: &'static str,
        version: InertiaVersion,
        template_path: &'static str,
        template_resolver: TemplateResolver
    ) -> Self {
        Self::instantiate(url, template_path, version, template_resolver, None)
    }

    /// Initializes an instance of [`Inertia`] struct with server-side rendering enabled.
    ///
    /// # Arguments
    /// * `url`                 -   A valid [href] of the current application
    /// * `version`             -   The current asset version of the application.
    ///                             See [Asset versioning] for more details.
    /// * `template_path        -   The path for the root html template.
    /// * `template_resolver`   -   A function that renders the given root template html. Check
    ///                             more details at [`Inertia::template_resolver`] doc string.
    /// * `ssr_host`            -   A [`Uri`] of the Ssr server.
    /// * `ssr_port`            -   A `u16` number representing the port of the Ssr server.
    ///
    /// # Errors
    /// Returns an [`InertiaError::SsrError`] if it fails to connect to the server.
    ///
    /// [`Inertia`]: Inertia
    /// [href]: https://developer.mozilla.org/en-US/docs/Web/API/Location
    /// [Asset versioning]: https://inertiajs.com/asset-versioning
    /// [`Inertia::template_resolver`]: Inertia
    /// [`InertiaError::SsrError`]: InertiaError::SsrError
    pub async fn new_with_ssr(
        url: &'static str,
        version: InertiaVersion,
        template_path: &'static str,
        template_resolver: TemplateResolver,
        ssr_host: hyper::Uri,
        ssr_port: u16,
    ) -> Result<Self, InertiaError> {
        let address = format!("{}:{}", ssr_host, ssr_port);
        let stream = match tokio::net::TcpStream::connect(address).await {
            Err(err) => return Err(InertiaError::SsrError(
                inertia_err_msg(format!("Failed to start ssr tcp connection: {}", err.to_string()))
            )),
            Ok(stream) => stream,
        };

        let io = hyper_util::rt::TokioIo::new(stream);

        let (mut sender, conn) =
            match hyper::client::conn::http1::handshake(io).await {
                Err(err) => return Err(InertiaError::SsrError(
                    inertia_err_msg(format!("Failed to execute http handshake: {}", err.to_string()))
                )),
                Ok(value) => value,
            };

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                inertia_panic(format!("Could not connect to Ssr client: {:?}", err));
            }
        });

        let client = SsrClient {
          sender,
        };

        Ok(Self::instantiate(url, template_path, version, template_resolver, Some(client)))
    }

    fn instantiate (
        url: &'static str,
        template_path: &'static str,
        version: InertiaVersion,
        template_resolver: TemplateResolver,
        ssr_client: Option<SsrClient<String>>
    ) -> Self {
        let version = match version {
            InertiaVersion::Literal(v) => v.leak(),
            InertiaVersion::Resolver(resolver) => resolver().leak(),
        };

        Self {
            url,
            template_path,
            version,
            template_resolver,
            ssr_client,
            custom_view_data: Map::new(),
        }
    }

    pub fn get_view_data_mut(&mut self) -> &Map<String, Value> {
        &mut self.custom_view_data
    }
}
