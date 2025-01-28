use crate::{template_resolver::TemplateResolver, InertiaVersion, SsrClient};

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
/// * `template_resolver`       -   A valid template resolver. Check [Template Resolvers] chapter for more details.
/// * `with_ssr`                -   Whether Server-side Rendering should be enabled or not.
/// * `custom_ssr_client`       -   An [`Option<SsrClient>`] with the Inertia Server address.
///                                 If `None` is given, `SsrClient::default` will
///                                 be used.
/// * `encrypt_history`         -   Whether to encrypt or not the session. Refer to [History encryption]
///                                 for more details.
///
/// [Template Resolvers]: https://kaiofelps.github.io/inertia-rust/advanced/template_resolvers.html
/// [Flash Messages and Validation Errors]: https://kaiofelps.github.io/inertia-rust/advanced/flash-messages.html
/// [History encryption]: https://inertiajs.com/history-encryption
pub struct InertiaConfig<V>
where
    V: ToString,
{
    pub url: &'static str,
    pub version: InertiaVersion<V>,
    pub template_resolver: Box<dyn TemplateResolver + Send + Sync>,
    pub with_ssr: bool,
    pub custom_ssr_client: Option<SsrClient>,
    pub encrypt_history: bool,
}

impl<V> InertiaConfig<V>
where
    V: ToString,
{
    /// Instatiates a new InertiaConfigBuilder instance. It must be configured using a fluent syntax.
    ///
    /// # Examples
    /// ```rust
    /// use inertia_rust::{InertiaVersion, InertiaConfig};
    /// # use inertia_rust::{ViewData, InertiaError, template_resolvers::TemplateResolver};
    /// #
    /// #   struct YourTemplateResolver;
    /// #
    /// #   #[async_trait::async_trait(?Send)]
    /// #   impl TemplateResolver for YourTemplateResolver {
    /// #       async fn resolve_template(
    /// #           &self,
    /// #           view_data: ViewData<'_>,
    /// #       ) -> Result<String, InertiaError> {
    /// #           // import the layout root and render it using your template engine
    /// #           // lets pretend we rendered it, so it ended up being the html output below!
    /// #           Ok("<h1>my rendered page!</h1>".to_string())
    /// #       }
    /// #   }
    /// #
    /// let inertia_config = InertiaConfig::builder()
    ///     .set_url("http://localhost:8080")
    ///     .set_version(InertiaVersion::Literal("v1"))
    ///     .set_template_resolver(Box::new(YourTemplateResolver))
    ///     .build();
    /// ```
    pub fn builder() -> InertiaConfigBuilder<V> {
        InertiaConfigBuilder::new()
    }
}

pub struct InertiaConfigBuilder<V>
where
    V: ToString,
{
    pub url: Option<&'static str>,
    pub version: Option<InertiaVersion<V>>,
    pub template_resolver: Option<Box<dyn TemplateResolver + Send + Sync>>,
    pub with_ssr: bool,
    pub custom_ssr_client: Option<SsrClient>,
    pub encrypt_history: bool,
}

impl<V> Default for InertiaConfigBuilder<V>
where
    V: ToString,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V> InertiaConfigBuilder<V>
where
    V: ToString,
{
    /// Instatiates a new InertiaConfigBuilder instance. It must be configured using a fluent syntax.
    ///
    /// # Examples
    /// ```rust
    /// use inertia_rust::{InertiaConfigBuilder, InertiaVersion};
    ///
    /// # use inertia_rust::{template_resolvers::TemplateResolver, ViewData, InertiaError};
    /// #   struct YourTemplateResolver;
    /// #
    /// #   #[async_trait::async_trait(?Send)]
    /// #   impl TemplateResolver for YourTemplateResolver {
    /// #       async fn resolve_template(
    /// #           &self,
    /// #           view_data: ViewData<'_>,
    /// #       ) -> Result<String, InertiaError> {
    /// #           // import the layout root and render it using your template engine
    /// #           // lets pretend we rendered it, so it ended up being the html output below!
    /// #           Ok("<h1>my rendered page!</h1>".to_string())
    /// #       }
    /// #   }
    /// #
    /// let inertia_config = InertiaConfigBuilder::new()
    ///     .set_url("http://localhost:8080")
    ///     .set_version(InertiaVersion::Literal("v1"))
    ///     .set_template_resolver(Box::new(YourTemplateResolver))
    ///     .build();
    /// ```
    pub fn new() -> Self {
        Self {
            url: None,
            version: None,
            template_resolver: None,
            with_ssr: false,
            custom_ssr_client: None,
            encrypt_history: false,
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

    pub fn set_template_resolver(
        mut self,
        template_resolver: Box<dyn TemplateResolver + Send + Sync>,
    ) -> Self {
        self.template_resolver = Some(template_resolver);
        self
    }

    pub fn enable_ssr(mut self) -> Self {
        self.with_ssr = true;
        self
    }

    pub fn encrypt_history(mut self) -> Self {
        self.encrypt_history = true;
        self
    }

    /// Compile the current `InertiaConfigBuilder` into a valid `InertiaConfig` struct.
    ///
    /// # Panics
    /// Panics if any of the following fields equal [`None`]:
    /// * `url`
    /// * `template_resolver`
    /// * `version`
    pub fn build(self) -> InertiaConfig<V> {
        if self.url.is_none() {
            panic!(
            "[InertiaConfigBuilder] 'url' is a mandatory field and InertiaConfigBuilder cannot build without it.");
        }

        if self.template_resolver.is_none() {
            panic!(
            "[InertiaConfigBuilder] 'template_resolver' is a mandatory field and InertiaConfigBuilder cannot build without it.");
        }

        if self.version.is_none() {
            panic!(
            "[InertiaConfigBuilder] 'version' is a mandatory field and InertiaConfigBuilder cannot build without it.");
        }

        InertiaConfig {
            url: self.url.unwrap(),
            template_resolver: self.template_resolver.unwrap(),
            version: self.version.unwrap(),
            with_ssr: self.with_ssr,
            custom_ssr_client: self.custom_ssr_client,
            encrypt_history: self.encrypt_history,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{template_resolver::TemplateResolver, InertiaError, InertiaVersion, ViewData};
    use std::panic;

    use super::{InertiaConfig, InertiaConfigBuilder};

    // region: --- Mocks

    #[derive(PartialEq, Eq)]
    struct MyTemplateResolver;

    #[async_trait::async_trait(?Send)]
    impl TemplateResolver for MyTemplateResolver {
        async fn resolve_template(&self, _view_data: ViewData<'_>) -> Result<String, InertiaError> {
            Ok("".to_string())
        }
    }

    // endregion: --- Mocks

    // region: --- Tests

    #[test]
    fn builder_panics_if_critical_fields_are_unset() {
        // region: --- builders
        let build_totally_empty = panic::catch_unwind(move || {
            InertiaConfigBuilder::<&str>::new().build();
        });

        let build_without_url = panic::catch_unwind(move || {
            InertiaConfigBuilder::<&str>::new()
                .set_template_resolver(Box::new(MyTemplateResolver))
                .set_version(InertiaVersion::Literal("v1"))
                .build()
        });

        let build_without_template_resolver = panic::catch_unwind(move || {
            InertiaConfigBuilder::<&str>::new()
                .set_url("foo")
                .set_version(InertiaVersion::Literal("v1"))
                .build()
        });

        let build_without_version = panic::catch_unwind(move || {
            InertiaConfigBuilder::<&str>::new()
                .set_url("foo")
                .set_template_resolver(Box::new(MyTemplateResolver))
                .build()
        });

        let build_with_critical_fields_filled = panic::catch_unwind(move || {
            InertiaConfigBuilder::<&str>::new()
                .set_url("foo")
                .set_template_resolver(Box::new(MyTemplateResolver))
                .set_version(InertiaVersion::Literal("v1"))
                .build()
        });
        // endregion: --- builders

        assert!(build_totally_empty.is_err());
        assert!(build_without_url.is_err());
        assert!(build_without_template_resolver.is_err());
        assert!(build_without_version.is_err());
        assert!(build_with_critical_fields_filled.is_ok());
    }

    #[test]
    fn builder_builds_correctly() {
        let with_builder = InertiaConfigBuilder::<&str>::new()
            .set_url("foo")
            .set_template_resolver(Box::new(MyTemplateResolver))
            .set_version(InertiaVersion::Literal("v1"))
            .build();

        let directly_initialized = InertiaConfig {
            url: "foo",
            template_resolver: Box::new(MyTemplateResolver),
            version: InertiaVersion::Literal("v1"),
            with_ssr: false,
            custom_ssr_client: None,
            encrypt_history: false,
        };

        assert_eq!(&with_builder.url, &directly_initialized.url);
        assert_eq!(
            &with_builder.version.resolve(),
            &directly_initialized.version.resolve()
        );
        assert_eq!(&with_builder.with_ssr, &directly_initialized.with_ssr);
        assert_eq!(
            &with_builder.custom_ssr_client,
            &directly_initialized.custom_ssr_client
        );
    }

    // endregion: --- Tests
}
