use std::future::Future;
use std::io;
use std::pin::Pin;

use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use crate::config::InertiaConfig;
use crate::{InertiaError, InertiaPage, InertiaSSRPage};
use crate::node_process::NodeJsProc;
use crate::props::InertiaProps;
use crate::req_type::InertiaRequestType;

#[allow(unused)] pub const X_INERTIA: &str = "x-inertia";
#[allow(unused)] pub const X_INERTIA_LOCATION: &str = "x-inertia-location";
#[allow(unused)] pub const X_INERTIA_VERSION: &str = "x-inertia-version";
#[allow(unused)] pub const X_INERTIA_PARTIAL_COMPONENT: &str = "x-inertia-partial-component";
#[allow(unused)] pub const X_INERTIA_PARTIAL_DATA: &str = "x-inertia-partial-data";
#[allow(unused)] pub const X_INERTIA_PARTIAL_EXCEPT: &str = "x-inertia-partial-except";

/// The javascript component name.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Component(pub String);

/// InertiaResponder trait defines methods that every provider
/// must implement. For instance, T may be a sort of actix-web Responder,
/// if "actix" feature is passed with the --feature flag or with the
/// feature field in the cargo toml.
#[async_trait(?Send)] // it's `?Send` because some frameworks like Actix won't require requests to be thread-safe
pub trait InertiaResponder<TResponder, THttpRequest> {
    /// Renders an Inertia Page as an HTTP response.
    ///
    /// # Arguments
    /// * `req`         -   The HTTP request.
    /// * `component`   -   The page javascript component name to be rendered by the
    ///                     client-side adapter.
    async fn render(&self, req: &THttpRequest, component: Component) -> Result<TResponder, InertiaError>;

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
    async fn render_with_props(&self, req: &THttpRequest, component: Component, props: InertiaProps) -> Result<TResponder, InertiaError>;

    /// Provokes a client-side redirect to an extern URL.
    ///
    /// # Arguments
    /// * `req`     - A reference to the HTTP request.
    /// * `url`     - The URL to be redirected to.
    fn location(req: &THttpRequest, url: &str) -> TResponder;
}

pub trait InertiaErrMapper<TResponder, THttpResponse> {
    fn map_inertia_err(self) -> TResponder;
}

/// Defines some helper methods to be implemented to HttpRequests from the
/// library opted by the cargo feature.
pub(crate) trait InertiaHttpRequest {
    fn is_inertia_request(&self) -> bool;

    fn get_request_type(&self) -> Result<InertiaRequestType, InertiaError>;

    fn check_inertia_version(&self, current_version: &str) -> bool;
}

pub enum InertiaVersion<T> where  T: ToString {
    Literal(T),
    Resolver(Box<dyn FnOnce() -> T>)
}

impl<T> InertiaVersion<T> where T: ToString {
    pub fn resolve(self) -> &'static str {
        match self {
            InertiaVersion::Literal(v) => v.to_string().leak(),
            InertiaVersion::Resolver(resolver) => resolver().to_string().leak(),
        }
    }
}

/// View Data is a struct containing props to be used by the root template.
pub struct ViewData {
    pub page: InertiaPage,
    pub ssr_page: Option<InertiaSSRPage>,
    pub custom_props: Map<String, Value>
}

pub type TemplateResolverOutput = Pin<Box<dyn Future<Output = Result<String, InertiaError>> + Send + Sync + 'static>>;
pub(crate) type TemplateResolver<T> = &'static (dyn Fn(&'static str, ViewData, &'static T) -> TemplateResolverOutput + Send + Sync + 'static);

#[derive(PartialEq, Debug)]
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
pub struct Inertia<T> where T : 'static {
    /// URL used between redirects and responses generation, i.g. "https://myapp.com".
    #[allow(unused)]
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
    /// Returns an [`InertiaError::RenderError`] if it fails to render the html.
    ///
    /// # Return
    /// The return must be the template rendered to HTML. It will be sent as response to full
    /// requests.
    pub(crate) template_resolver: TemplateResolver<T>,
    /// The data to provide to template resolver
    pub(crate) template_resolver_data: &'static T,
    /// Address of Inertia local render server. Will be used by Inertia to perform ssr.
    pub(crate) ssr_url: Option<Url>,
    /// Extra data to be passed to the root template.
    pub(crate) custom_view_data: Map<String, Value>
}

impl<T> Inertia<T> where T : 'static {
    /// Initializes an instance of [`Inertia`] struct.
    ///
    /// # Arguments
    /// * `url`                     -   A valid [href] of the current application
    /// * `version`                 -   The current asset version of the application.
    ///                                 See [Asset versioning] for more details.
    /// * `template_path`           -   The path for the root html template.
    /// * `template_resolver`       -   A function that renders the given root template html. Check
    ///                                 more details at [`Inertia::template_resolver`] doc string.
    /// * `template_resolver_data`  -   The third parameter of your template resolver. Inertia will
    ///                                 pass it by reference when calling the resolver function.
    ///                                 If you don't plan to use it, just pass an empty tuple (both here
    ///                                 and in your template resolver).
    ///
    ///  # Errors
    /// Returns an [`InertiaError::SsrError`] if it fails to connect to the server.
    pub fn new<V>(config: InertiaConfig<T, V>) -> Result<Self, io::Error>
        where V: ToString
    {
        let version = config.version.resolve();
        let ssr_url = match config.with_ssr {
            false => None,
            true => {
                let client: SsrClient = config.custom_ssr_client.unwrap_or_default();

                let ssr_url = if client.host.contains("://") {
                    format!("{}:{}", client.host, client.port)
                } else {
                    format!("http://{}:{}", client.host, client.port)
                };

                match Url::parse(&ssr_url) {
                    Err(err) => {
                        let inertia_err = InertiaError::SsrError(format!("Failed to parse Inertia Server url: {}", err));
                        return Err(inertia_err.to_io_error());
                    },
                    Ok(url) => Some(url),
                }
            }
        };

        Ok(Self {
            url: config.url,
            template_path: config.template_path,
            version,
            template_resolver: config.template_resolver,
            template_resolver_data: config.template_resolver_data,
            ssr_url,
            custom_view_data: config.view_data.unwrap_or_default(),
        })
    }

    pub fn get_view_data_mut(&mut self) -> &Map<String, Value> {
        &mut self.custom_view_data
    }

    /// Instantiates a [`NodeJsProc`] by calling [`NodeJsProc::start`] with the given path and the
    /// inertia `ssr_url` as server url.
    ///
    /// # Arguments
    /// * `server_file_path`    - The path to the server javascript file. E.g. "dist/server/ssr.js".
    ///
    /// # Errors
    /// Will return an [`InertiaError`] if ssr is not enabled or if something goes wrong on setting
    /// the node.js server up (if your machine do not have node installed, for example).
    ///
    /// # Return
    /// Returns a [`NodeJsProc`] instance.
    ///
    /// # Example
    /// ```rust
    /// use inertia_rust::node_process::NodeJsProc;
    /// use inertia_rust::{
    ///     Inertia,
    ///     InertiaVersion,
    ///     InertiaError,
    ///     ViewData,
    ///     TemplateResolverOutput,
    ///     InertiaConfig
    /// };
    /// use std::pin::Pin;
    /// use std::future::Future;
    ///
    /// async fn server() {
    ///     // note that this is the async function and the actual resolver
    ///     async fn _resolver(
    ///         path: &'static str, // "www/index.html"
    ///         view_data: ViewData,
    ///         _data: &'static ()
    ///     ) -> Result<String, InertiaError> {
    ///         // import the layout root and render it using your template engine
    ///         // lets pretend we rendered it, so it ended up being the html output below!
    ///         Ok("<h1>my rendered page!</h1>".to_string())
    ///     }
    ///
    ///     // a wrapper for the resolver, so that it can be stored inside the Inertia struct
    ///     fn resolver(
    ///         path: &'static str,
    ///         view_data: ViewData,
    ///         _data: &'static ()
    ///     ) -> TemplateResolverOutput {
    ///         Box::pin(_resolver(path, view_data, _data))
    ///     }
    /// 
    ///     let inertia = Inertia::new(
    ///         InertiaConfig::builder()
    ///             .set_url("https://www.my-web-app.com")
    ///             .set_version(InertiaVersion::Literal("my-assets-version"))
    ///             .set_template_resolver(&resolver)
    ///             .set_template_path("www/index.html")
    ///             .set_template_resolver_data(&())
    ///             .build()
    ///     )
    ///     .unwrap();
    ///
    ///     let node: Result<NodeJsProc, std::io::Error> = inertia.start_node_server("dist/server/ssr.js".into());
    ///     if node.is_err() {
    ///         let err = node.unwrap_err();
    ///         panic!("Failed to start inertia ssr server: {:?}", err);
    ///     }
    ///
    ///     let node = node.unwrap();
    ///
    ///     // starts your server here, using inertia.
    ///     // httpserver().await; or something like this
    ///
    ///     let _ = node.kill(); // don't forget to kill the node.js process on shutdown
    /// }
    /// ```
    pub fn start_node_server(&self, server_file_path: String) -> Result<NodeJsProc, io::Error> {
        if self.ssr_url.is_none() {
            let inertia_err: InertiaError = InertiaError::SsrError("Ssr is not enabled and, hence, a ssr server cannot be raised.".into());
            return Err(inertia_err.to_io_error());
        }

        let node = NodeJsProc::start(server_file_path, self.ssr_url.as_ref().unwrap());
        match node {
            Err(err) => Err(InertiaError::NodeJsError(err).to_io_error()),
            Ok(process) => Ok(process)
        }
    }
}
