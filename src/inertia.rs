use std::future::Future;
use std::pin::Pin;
use async_trait::async_trait;
use reqwest::Url;
use serde::Serialize;
use serde_json::{Map, Value};
use crate::{InertiaError, InertiaPage, InertiaSSRPage};
use crate::props::InertiaProps;
use crate::req_type::InertiaRequestType;

#[allow(unused)] pub const X_INERTIA: &'static str = "x-inertia";
#[allow(unused)] pub const X_INERTIA_LOCATION: &'static str = "x-inertia-location";
#[allow(unused)] pub const X_INERTIA_VERSION: &'static str = "x-inertia-version";
#[allow(unused)] pub const X_INERTIA_PARTIAL_COMPONENT: &'static str = "x-inertia-partial-component";
#[allow(unused)] pub const X_INERTIA_PARTIAL_DATA: &'static str = "x-inertia-partial-data";
#[allow(unused)] pub const X_INERTIA_PARTIAL_EXCEPT: &'static str = "x-inertia-partial-except";

/// The javascript component name.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
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
pub struct ViewData<'lf> {
    pub page: InertiaPage<'lf>,
    pub ssr_page: Option<InertiaSSRPage>,
    pub custom_props: Map<String, Value>
}

pub(crate) type TemplateResolver = fn(&'_ str, ViewData) -> Pin<Box<dyn Future<Output = Result<String, InertiaError>> + Send + 'static>>;

pub struct SsrClient {
    pub(crate) host: &'static str,
    pub(crate) port: u16,
}

impl SsrClient {
    /// Generates a new custom `SsrClient` struct. Unless you really need to change the ssr server
    /// url, it is preferred to use `SsrClient::Default` for generating a new SsrClient struct.
    ///
    /// # Arguments
    /// * `host`    -   The host of the server (normally, `127.0.0.1`, since it should run locally
    /// * `port`    -   The server port
    pub fn new(host: &'static str, port: u16) -> Self {
        Self { host, port }
    }
}

impl Default for SsrClient {
    fn default() -> Self {
        Self {
            host: "127.0.0.1",
            port: 13714
        }
    }
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
    /// Address of Inertia local render server. Will be used by Inertia to perform ssr.
    pub(crate) ssr_url: Option<Url>,
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
    /// Run the command to raise the ssr server, or else no ssr will be done. By the way, check the
    /// GitHub repository readme to find the current command.
    ///
    /// # Arguments
    /// * `url`                 -   A valid [href] of the current application
    /// * `version`             -   The current asset version of the application.
    ///                             See [Asset versioning] for more details.
    /// * `template_path        -   The path for the root html template.
    /// * `template_resolver`   -   A function that renders the given root template html. Check
    ///                             more details at [`Inertia::template_resolver`] doc string.
    /// * `custom_client`       -   An [`Option<SsrClient>`] with the Inertia Server address.
    ///                             If `None` is passed to the parameters, `SsrClient::default` will
    ///                             be used.
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
        custom_client: Option<SsrClient>,
    ) -> Result<Self, InertiaError> {
        let client: SsrClient = custom_client.unwrap_or_else(|| SsrClient::default());

        let ssr_url = if client.host.contains("://") {
            format!("{}:{}", client.host, client.port)
        } else {
            format!("http://{}:{}", client.host, client.port)
        };

        let ssr_url = match Url::parse(&ssr_url) {
            Err(err) => return Err(InertiaError::SsrError(format!("Failed to parse Inertia Server url: {}", err))),
            Ok(url) => url,
        };

        Ok(Self::instantiate(url, template_path, version, template_resolver, Some(ssr_url)))
    }

    fn instantiate (
        url: &'static str,
        template_path: &'static str,
        version: InertiaVersion,
        template_resolver: TemplateResolver,
        ssr_url: Option<Url>
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
            ssr_url,
            custom_view_data: Map::new(),
        }
    }

    pub fn get_view_data_mut(&mut self) -> &Map<String, Value> {
        &mut self.custom_view_data
    }
}
