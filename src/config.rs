use crate::{inertia::TemplateResolver, InertiaVersion, SsrClient};
use serde_json::{Map, Value};

/// A configuration struct for initializing Inertia. You can directly fill the struct or use
/// the builder fluent syntax by calling `InertiaConfig::builder()`, and finally `InertiaConfig::build()`.
///
/// Note that, even with builder, most of fields are mandatory and trying to build without filling them
/// will cause your application to `panic!`
///
/// * `url`                     -   A valid [href](https://developer.mozilla.org/en-US/docs/Web/API/Location)
///                                 of the currentapplication
/// * `version`                 -   The current asset version of the application.
///                                 See [Asset versioning](https://inertiajs.com/asset-versioning) for more
///                                 details.
/// * `template_path`           -   The path for the root html template.
/// * `template_resolver`       -   A function that renders the given root template html. Check
///                                 more details at [`Inertia::template_resolver`] document string.
/// * `template_resolver_data`  -   The third parameter of your template resolver. Inertia will
///                                 pass it by reference when calling the resolver function.
///                                 If you don't plan to use it, just pass an empty tuple (both here
///                                 and in your template resolver).
/// * `with_ssr`                -   Whether Server-side Rendering should be enabled.
/// * `custom_ssr_client`       -   An [`Option<SsrClient>`] with the Inertia Server address.
///                                 If `None` is given, `SsrClient::default` will
///                                 be used.
/// * `view_data`               -   Optional view data to be passed to the root template. It must be
///                                 handled by the provided `template_resolver`.
///
/// [`Inertia::template_resolver`]: crate::inertia::Inertia
pub struct InertiaConfig<T, V>
where
    T: 'static,
    V: ToString,
{
    pub url: &'static str,
    pub version: InertiaVersion<V>,
    pub template_path: &'static str,
    pub template_resolver: TemplateResolver<T>,
    pub template_resolver_data: &'static T,
    pub with_ssr: bool,
    pub custom_ssr_client: Option<SsrClient>,
    pub view_data: Option<Map<String, Value>>,
}

impl<T, V> InertiaConfig<T, V>
where
    T: 'static,
    V: ToString,
{
    /// Instatiates a new InertiaConfigBuilder instance. It must be configured using a fluent syntax.
    ///
    /// # Examples
    /// ```rust
    /// use inertia_rust::{InertiaVersion, InertiaConfig};
    /// # use inertia_rust::{TemplateResolverOutput, ViewData, InertiaError};
    /// # async fn _your_template_resolver(_template_path: &str, _view_data: ViewData) -> Result<String, InertiaError> {
    /// #     return Ok("".to_string());
    /// # }
    /// #
    /// # pub fn your_template_resolver(template_path: &'static str, view_data: ViewData, _data: &()) -> TemplateResolverOutput {
    /// #     Box::pin(_your_template_resolver(template_path, view_data))
    /// # }
    /// #
    /// let inertia_config = InertiaConfig::builder()
    ///     .set_url("http://localhost:8080")
    ///     .set_version(InertiaVersion::Literal("v1"))
    ///     .set_template_path("path/to/template.html")
    ///     .set_template_resolver(&your_template_resolver)
    ///     .set_template_resolver_data(&())
    ///     .build();
    /// ```
    pub fn builder() -> InertiaConfigBuilder<T, V> {
        InertiaConfigBuilder::new()
    }
}

pub struct InertiaConfigBuilder<T, V>
where
    T: 'static,
    V: ToString,
{
    pub url: Option<&'static str>,
    pub version: Option<InertiaVersion<V>>,
    pub template_path: Option<&'static str>,
    pub template_resolver: Option<TemplateResolver<T>>,
    pub template_resolver_data: Option<&'static T>,
    pub with_ssr: bool,
    pub custom_ssr_client: Option<SsrClient>,
    pub view_data: Option<Map<String, Value>>,
}

impl<T, V> Default for InertiaConfigBuilder<T, V>
where
    T: 'static,
    V: ToString,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, V> InertiaConfigBuilder<T, V>
where
    T: 'static,
    V: ToString,
{
    /// Instatiates a new InertiaConfigBuilder instance. It must be configured using a fluent syntax.
    ///
    /// # Examples
    /// ```rust
    /// use inertia_rust::{InertiaConfigBuilder, InertiaVersion};
    ///
    /// # use inertia_rust::{TemplateResolverOutput, ViewData, InertiaError};
    /// # async fn _your_template_resolver(_template_path: &str, _view_data: ViewData) -> Result<String, InertiaError> {
    /// #     return Ok("".to_string());
    /// # }
    /// # pub fn your_template_resolver(template_path: &'static str, view_data: ViewData, _data: &()) -> TemplateResolverOutput {
    /// #     Box::pin(_your_template_resolver(template_path, view_data))
    /// # }
    /// #
    /// let inertia_config = InertiaConfigBuilder::new()
    ///     .set_url("http://localhost:8080")
    ///     .set_version(InertiaVersion::Literal("v1"))
    ///     .set_template_path("path/to/template.html")
    ///     .set_template_resolver(&your_template_resolver)
    ///     .set_template_resolver_data(&())
    ///     .build();
    /// ```
    pub fn new() -> Self {
        Self {
            url: None,
            version: None,
            template_path: None,
            template_resolver: None,
            template_resolver_data: None,
            view_data: None,
            with_ssr: false,
            custom_ssr_client: None,
        }
    }

    pub fn set_ssr_client(mut self, ssr_client: SsrClient) -> Self {
        self.custom_ssr_client = Some(ssr_client);
        self
    }

    pub fn set_url(mut self, url: &'static str) -> Self {
        self.url = Some(url);
        self
    }

    pub fn set_version(mut self, version: InertiaVersion<V>) -> Self {
        self.version = Some(version);
        self
    }

    pub fn set_template_path(mut self, template_path: &'static str) -> Self {
        self.template_path = Some(template_path);
        self
    }

    pub fn set_template_resolver(mut self, template_resolver: TemplateResolver<T>) -> Self {
        self.template_resolver = Some(template_resolver);
        self
    }

    pub fn set_template_resolver_data(mut self, data: &'static T) -> Self {
        self.template_resolver_data = Some(data);
        self
    }

    pub fn set_view_data(mut self, view_data: Map<String, Value>) -> Self {
        self.view_data = Some(view_data);
        self
    }

    pub fn enable_ssr(mut self) -> Self {
        self.with_ssr = true;
        self
    }

    /// Compile the current `InertiaConfigBuilder` into a valid `InertiaConfig` struct.
    ///
    /// # Panics
    /// Panics if any of the following fields equal [`None`]:
    /// * `url`
    /// * `template_path`
    /// * `template_resolver`
    /// * `template_resolver_data`
    /// * `version`
    pub fn build(self) -> InertiaConfig<T, V> {
        if self.url.is_none() {
            panic!(
            "[InertiaConfigBuilder] 'url' is a mandatory field and InertiaConfigBuilder cannot build without it.");
        }

        if self.template_path.is_none() {
            panic!(
            "[InertiaConfigBuilder] 'template_path' is a mandatory field and InertiaConfigBuilder cannot build without it.");
        }

        if self.template_resolver.is_none() {
            panic!(
            "[InertiaConfigBuilder] 'template_resolver' is a mandatory field and InertiaConfigBuilder cannot build without it.");
        }

        if self.template_resolver_data.is_none() {
            panic!(
            "[InertiaConfigBuilder] 'template_resolver_data' is a mandatory field and InertiaConfigBuilder cannot build without it.");
        }

        if self.version.is_none() {
            panic!(
            "[InertiaConfigBuilder] 'version' is a mandatory field and InertiaConfigBuilder cannot build without it.");
        }

        InertiaConfig {
            url: self.url.unwrap(),
            template_path: self.template_path.unwrap(),
            template_resolver: self.template_resolver.unwrap(),
            template_resolver_data: self.template_resolver_data.unwrap(),
            version: self.version.unwrap(),
            view_data: self.view_data,
            with_ssr: self.with_ssr,
            custom_ssr_client: self.custom_ssr_client,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{InertiaError, InertiaVersion, TemplateResolverOutput, ViewData};
    use std::panic;

    use super::{InertiaConfig, InertiaConfigBuilder};

    // region: --- Mocks

    async fn _mocked_resolver(
        _template_path: &str,
        _view_data: ViewData,
    ) -> Result<String, InertiaError> {
        Ok("".to_string())
    }

    pub fn mocked_resolver(
        template_path: &'static str,
        view_data: ViewData,
        _data: &(),
    ) -> TemplateResolverOutput {
        Box::pin(_mocked_resolver(template_path, view_data))
    }

    // endregion: --- Mocks

    // region: --- Tests

    #[test]
    fn builder_panics_if_critical_fields_are_unset() {
        // region: --- builders
        let build_totally_empty = panic::catch_unwind(move || {
            InertiaConfigBuilder::<(), &str>::new().build();
        });

        let build_without_url = panic::catch_unwind(move || {
            InertiaConfigBuilder::<(), &str>::new()
                .set_template_resolver(&mocked_resolver)
                .set_template_path("path")
                .set_template_resolver_data(&())
                .set_version(InertiaVersion::Literal("v1"))
                .build()
        });

        let build_without_template_resolver = panic::catch_unwind(move || {
            InertiaConfigBuilder::<(), &str>::new()
                .set_url("foo")
                .set_template_path("path")
                .set_template_resolver_data(&())
                .set_version(InertiaVersion::Literal("v1"))
                .build()
        });

        let build_without_template_path = panic::catch_unwind(move || {
            InertiaConfigBuilder::<(), &str>::new()
                .set_url("foo")
                .set_template_resolver(&mocked_resolver)
                .set_template_resolver_data(&())
                .set_version(InertiaVersion::Literal("v1"))
                .build()
        });

        let build_without_template_data = panic::catch_unwind(move || {
            InertiaConfigBuilder::<(), &str>::new()
                .set_url("foo")
                .set_template_resolver(&mocked_resolver)
                .set_template_path("path")
                .set_version(InertiaVersion::Literal("v1"))
                .build()
        });

        let build_without_version = panic::catch_unwind(move || {
            InertiaConfigBuilder::<(), &str>::new()
                .set_url("foo")
                .set_template_resolver(&mocked_resolver)
                .set_template_path("path")
                .set_template_resolver_data(&())
                .build()
        });

        let build_with_critical_fields_filled = panic::catch_unwind(move || {
            InertiaConfigBuilder::<(), &str>::new()
                .set_url("foo")
                .set_template_resolver(&mocked_resolver)
                .set_template_path("path")
                .set_template_resolver_data(&())
                .set_version(InertiaVersion::Literal("v1"))
                .build()
        });
        // endregion: --- builders

        assert!(build_totally_empty.is_err());
        assert!(build_without_url.is_err());
        assert!(build_without_template_resolver.is_err());
        assert!(build_without_template_path.is_err());
        assert!(build_without_template_data.is_err());
        assert!(build_without_version.is_err());
        assert!(build_with_critical_fields_filled.is_ok());
    }

    #[test]
    fn builder_builds_correctly() {
        let with_builder = InertiaConfigBuilder::<(), &str>::new()
            .set_url("foo")
            .set_template_resolver(&mocked_resolver)
            .set_template_path("path")
            .set_template_resolver_data(&())
            .set_version(InertiaVersion::Literal("v1"))
            .build();

        let directly_initialized = InertiaConfig {
            url: "foo",
            template_resolver: &mocked_resolver,
            template_path: "path",
            template_resolver_data: &(),
            version: InertiaVersion::Literal("v1"),
            view_data: None,
            with_ssr: false,
            custom_ssr_client: None,
        };

        assert_eq!(&with_builder.url, &directly_initialized.url);
        assert_eq!(
            &with_builder.template_path,
            &directly_initialized.template_path
        );
        assert_eq!(
            &with_builder.template_resolver_data,
            &directly_initialized.template_resolver_data
        );
        assert_eq!(
            &with_builder.version.resolve(),
            &directly_initialized.version.resolve()
        );
        assert_eq!(&with_builder.view_data, &directly_initialized.view_data);
        assert_eq!(&with_builder.with_ssr, &directly_initialized.with_ssr);
        assert_eq!(
            &with_builder.custom_ssr_client,
            &directly_initialized.custom_ssr_client
        );
    }

    // endregion: --- Tests
}
