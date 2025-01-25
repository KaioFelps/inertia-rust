use crate::config::InertiaConfig;
use crate::node_process::NodeJsProc;
use crate::props::InertiaProps;
use crate::req_type::InertiaRequestType;
use crate::template_resolver::TemplateResolver;
use crate::{InertiaError, InertiaPage, InertiaSSRPage};
use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::io;

pub const X_INERTIA: &str = "x-inertia";
pub const X_INERTIA_LOCATION: &str = "x-inertia-location";
pub const X_INERTIA_VERSION: &str = "x-inertia-version";
pub const X_INERTIA_PARTIAL_COMPONENT: &str = "x-inertia-partial-component";
pub const X_INERTIA_PARTIAL_DATA: &str = "x-inertia-partial-data";
pub const X_INERTIA_PARTIAL_EXCEPT: &str = "x-inertia-partial-except";
pub const X_INERTIA_RESET: &str = "x-inertia-reset";
pub const X_INERTIA_ERROR_BAG: &str = "x-inertia-error-bag";

/// The javascript component name.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Component(pub String);

impl<T> From<T> for Component
where
    T: ToString,
{
    fn from(value: T) -> Self {
        Component(value.to_string())
    }
}

/// InertiaService trait define a method to be implemented to Inertia struct that allows
/// to generate simple routes directly, without needing to create a handler function.
pub trait InertiaService {
    /// Renders an Inertia component directly, without defining a specific handler function for it.
    ///
    /// # Arguments
    /// * `path`        -   The router path.
    /// * `component`   -   The component to be rendered.
    ///
    /// # Examples
    /// ```ignore
    /// use some_framework::App;
    ///
    /// App::new().inertia_route("/", "Index");
    /// ```
    fn inertia_route(self, path: &str, component: &'static str) -> Self;
}

/// InertiaResponder trait defines methods that every provider
/// must implement. For instance, T may be a sort of actix-web Responder,
/// if "actix" feature is passed with the --feature flag or with the
/// feature field in the cargo toml.
#[async_trait(?Send)] // it's `?Send` because some frameworks like Actix won't require requests to be thread-safe
pub trait InertiaResponder<TResponder, THttpRequest, TRedirect> {
    async fn inner_render<'b>(
        &'b self,
        req: &'b THttpRequest,
        component: Component,
        props: Option<InertiaProps<'b>>,
    ) -> Result<TResponder, InertiaError>;

    fn inner_back(&self, req: &THttpRequest) -> TRedirect;

    fn inner_back_with_errors<Key: ToString>(
        &self,
        req: &THttpRequest,
        errors: HashMap<Key, Value>,
    ) -> TRedirect;

    fn inner_location(req: &THttpRequest, url: &str) -> TResponder;

    fn inner_encrypt_history(req: &THttpRequest, encrypt: bool);

    fn inner_clear_history(req: &THttpRequest);
}

/// Defines some helper methods to be implemented to HttpRequests from the
/// library opted by the cargo feature.
pub(crate) trait InertiaHttpRequest {
    fn should_clear_history(&self) -> bool;

    fn should_encrypt_history(&self, default: bool) -> bool;

    fn get_merge_props_to_be_reset(&self) -> Vec<&str>;

    fn is_inertia_request(&self) -> bool;

    fn get_request_type(&self) -> Result<InertiaRequestType, InertiaError>;

    /// Checks if application assets version matches.
    /// If the request contains the inertia version header, it will be checked.
    /// Otherwise, it means it does not have outdated assets and can also pass.
    fn check_inertia_version(&self, current_version: &str) -> bool;
}

pub enum InertiaVersion<T>
where
    T: ToString,
{
    Literal(T),
    Resolver(Box<dyn FnOnce() -> T>),
}

impl<T> InertiaVersion<T>
where
    T: ToString,
{
    pub fn resolve(self) -> &'static str {
        match self {
            InertiaVersion::Literal(v) => v.to_string().leak(),
            InertiaVersion::Resolver(resolver) => resolver().to_string().leak(),
        }
    }
}

/// View Data is a struct containing props to be used by the root template.
pub struct ViewData<'a> {
    pub page: InertiaPage<'a>,
    pub ssr_page: Option<InertiaSSRPage>,
    pub custom_props: Map<String, Value>,
}

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
            port: 13714,
        }
    }
}

/// Inertia struct must be a singleton and initialized at the application bootstrap.
/// It is supposed to last during the whole application runtime.
///
/// Extra details of how to initialize and keep it is specific to the feature-opted http library.
pub struct Inertia {
    /// URL used between redirects and responses generation, i.g. "https://myapp.com".
    #[allow(unused)]
    pub(crate) url: &'static str,
    /// The path to find the root html template to render everything in.
    pub(crate) template_path: &'static str,
    /// The current assets version.
    pub(crate) version: &'static str,
    /// A struct that implements [TemplateResolver] trait.
    pub(crate) template_resolver: Box<dyn TemplateResolver + Send + Sync>,
    /// Address of Inertia local render server. Will be used by Inertia to perform ssr.
    pub(crate) ssr_url: Option<Url>,
    /// Whether to encrypt or not the page data stored in the browser history.
    pub(crate) encrypt_history: bool,
}

impl Inertia {
    /// Initializes an instance of [`Inertia`] struct.
    ///
    /// # Arguments
    /// * `config`  - A [`InertiaConfig`] instance.
    ///
    ///  # Errors
    /// Returns an [`InertiaError::SsrError`] if it fails to connect to the server.
    pub fn new<V>(config: InertiaConfig<V>) -> Result<Self, io::Error>
    where
        V: ToString,
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
                        let inertia_err = InertiaError::SsrError(format!(
                            "Failed to parse Inertia Server url: {}",
                            err
                        ));
                        return Err(inertia_err.to_io_error());
                    }
                    Ok(url) => Some(url),
                }
            }
        };

        Ok(Self {
            url: config.url,
            template_path: config.template_path,
            version,
            template_resolver: config.template_resolver,
            ssr_url,
            encrypt_history: config.encrypt_history,
        })
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
    ///     template_resolvers::TemplateResolver,
    ///     Inertia,
    ///     InertiaVersion,
    ///     InertiaError,
    ///     ViewData,
    ///     InertiaConfig
    /// };
    /// use std::pin::Pin;
    /// use std::future::Future;
    ///
    /// async fn server() {
    ///     struct MyTemplateResolver;
    ///
    ///     #[async_trait::async_trait(?Send)]
    ///     impl TemplateResolver for MyTemplateResolver {
    ///         async fn resolve_template(
    ///             &self,
    ///             template_path: &str,
    ///             view_data: ViewData<'_>,
    ///         ) -> Result<String, InertiaError> {
    ///             // import the layout root and render it using your template engine
    ///             // lets pretend we rendered it, so it ended up being the html output below!
    ///             Ok("<h1>my rendered page!</h1>".to_string())
    ///         }
    ///     }
    ///
    ///     let inertia = Inertia::new(
    ///         InertiaConfig::builder()
    ///             .set_url("https://www.my-web-app.com")
    ///             .set_version(InertiaVersion::Literal("my-assets-version"))
    ///             .set_template_resolver(Box::new(MyTemplateResolver))
    ///             .set_template_path("www/index.html")
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
            let inertia_err: InertiaError = InertiaError::SsrError(
                "Ssr is not enabled and, hence, a ssr server cannot be raised.".into(),
            );
            return Err(inertia_err.to_io_error());
        }

        let node = NodeJsProc::start(server_file_path, self.ssr_url.as_ref().unwrap());
        match node {
            Err(err) => Err(InertiaError::NodeJsError(err).to_io_error()),
            Ok(process) => Ok(process),
        }
    }

    pub fn get_url(&self) -> &'static str {
        self.url
    }

    pub fn get_version(&self) -> &'static str {
        self.version
    }

    pub fn get_ssr_url(&self) -> &Option<Url> {
        &self.ssr_url
    }
}
