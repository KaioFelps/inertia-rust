use async_trait::async_trait;
use handlebars::Handlebars;
use serde_json::json;
use vite_rust::{Vite, ViteMode};

use crate::{template_resolver::TemplateResolver, InertiaError, ViewData};

struct ViteStaticSerializedTags {
    pub scripts: &'static str,
    pub react_script: Option<&'static str>,
}

/// Vite HBS stands for Vite and Handlebars. This is a template resolver that also uses Vite Rust
/// as assets bundler, but, instead of simple regexes, it uses Handlebars as underlying template
/// engine for rendering the root template more efficiently.
pub struct ViteHBSTemplateResolver<'a> {
    pub vite: Vite,
    pub hbs: Handlebars<'a>,
    static_tags: ViteStaticSerializedTags,
}

impl<'a> ViteHBSTemplateResolver<'a> {
    pub fn new(vite: Vite, template_path: &'a str, dev_mode: bool) -> Result<Self, InertiaError> {
        let mut hbs = Handlebars::new();

        hbs.set_dev_mode(dev_mode);

        if let Err(err) = hbs.register_template_file("root", template_path) {
            return Err(InertiaError::RenderError(format!(
                "Failed to register template path: {}",
                err.reason()
            )));
        }

        let static_tags = Self::get_static_tags(&vite)?;

        Ok(Self {
            hbs,
            vite,
            static_tags,
        })
    }

    #[inline]
    pub fn builder() -> ViteHBSTemplateResolverBuilder<'a> {
        ViteHBSTemplateResolverBuilder::new()
    }

    #[inline]
    fn from_builder(builder: ViteHBSTemplateResolverBuilder<'a>) -> Result<Self, InertiaError> {
        let vite = builder.vite.expect("Vite hasn't been set. Please, ensure you've correctly configured the ViteHBSTemplateResolverBuilder.");
        let template_path = builder.template_path.expect("Template path hasn't been set. Please, ensure you've correctly configured the ViteHBSTemplateResolverBuilder.");

        if let Some(hbs) = builder.hbs {
            let static_tags = Self::get_static_tags(&vite)?;

            return Ok(Self {
                vite,
                hbs,
                static_tags,
            });
        }

        Self::new(vite, template_path, builder.dev_mode)
    }

    #[inline]
    fn get_static_tags(vite: &Vite) -> Result<ViteStaticSerializedTags, InertiaError> {
        let resolved_scripts = vite.get_resolved_vite_scripts().map_err(|err| {
            InertiaError::RenderError(format!("Failed to resolve vite scripts: {err}"))
        })?;

        Ok(ViteStaticSerializedTags {
            react_script: match vite.mode() {
                ViteMode::Manifest => None,
                ViteMode::Development => Some(Box::leak(vite.get_react_script().into_boxed_str())),
            },
            scripts: Box::leak(resolved_scripts.into_boxed_str()),
        })
    }
}

#[derive(Default)]
pub struct ViteHBSTemplateResolverBuilder<'a> {
    pub vite: Option<Vite>,
    pub hbs: Option<Handlebars<'a>>,
    pub template_path: Option<&'a str>,
    pub dev_mode: bool,
}

impl<'a> ViteHBSTemplateResolverBuilder<'a> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_custom_handlebars(mut self, hbs: Handlebars<'a>) -> Self {
        self.hbs = Some(hbs);
        self
    }

    pub fn set_vite(mut self, vite: Vite) -> Self {
        self.vite = Some(vite);
        self
    }

    pub fn set_template_path(mut self, template_path: &'a str) -> Self {
        self.template_path = Some(template_path);
        self
    }

    pub fn set_dev_mode(mut self, dev_mode: bool) -> Self {
        self.dev_mode = dev_mode;
        self
    }

    /// Turns the current builder into a valid `ViteHBSTemplateResolver`.
    ///
    /// # Panics
    /// Panics if `vite` or `template_path` hasn't been set.
    #[inline]
    pub fn build(self) -> Result<ViteHBSTemplateResolver<'a>, InertiaError> {
        ViteHBSTemplateResolver::from_builder(self)
    }
}

#[async_trait(?Send)]
impl TemplateResolver for ViteHBSTemplateResolver<'_> {
    async fn resolve_template(&self, view_data: ViewData<'_>) -> Result<String, InertiaError> {
        let (inertia_head, inertia_body) = match view_data.ssr_page {
            Some(ssr_page) => (ssr_page.get_head(), ssr_page.get_body()),
            None => {
                let stringified_page = match serde_json::to_string(&view_data.page) {
                    Ok(page) => page,
                    Err(_) => {
                        return Err(InertiaError::SerializationError(format!(
                            "Failed to serialize view_data.page: {:?}",
                            &view_data.page
                        )))
                    }
                };

                let container = format!("<div id='app' data-page='{}'></div>\n", stringified_page,);

                (String::new(), container)
            }
        };

        let data = json!({
            "inertia_head": inertia_head,
            "inertia_body": inertia_body,
            "page": view_data.page.get_props(),
            "view_data": view_data.custom_props,
            "vite": self.static_tags.scripts,
            "vite_react_refresh": self.static_tags.react_script
        });

        self.hbs.render("root", &data).map_err(|err| {
            InertiaError::RenderError(format!(
                "Could not render page due to Handlebars error: {}",
                err.reason()
            ))
        })
    }
}

#[cfg(test)]
mod test {
    use super::ViteHBSTemplateResolver;
    use crate::{
        hashmap, template_resolver::TemplateResolver, Inertia, InertiaConfigBuilder, InertiaPage,
        InertiaSSRPage, InertiaVersion, ViewData,
    };
    use serde_json::{json, Map};
    use vite_rust::{Vite, ViteConfig, ViteMode};

    async fn get_inertia<T: TemplateResolver + Send + Sync + 'static>(
        template_resolver: T,
    ) -> Inertia {
        Inertia::new(
            InertiaConfigBuilder::new()
                .set_url("http://localhost:3000")
                .set_version(InertiaVersion::Literal("1"))
                .set_template_resolver(Box::new(template_resolver))
                .build(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn render_on_dev_mode() {
        let vite = Vite::new(
            ViteConfig::new("tests/common/manifest.json", vec!["www/app.tsx"])
                .set_force_mode(ViteMode::Development),
        )
        .await
        .unwrap();

        let template_resolver =
            ViteHBSTemplateResolver::new(vite, "tests/common/root_layout.hbs", true).unwrap();

        let inertia = get_inertia(template_resolver).await;

        let page = InertiaPage {
            clear_history: false,
            encrypt_history: false,
            component: "Index".into(),
            deferred_props: None,
            merge_props: None,
            props: Map::from_iter(hashmap![
                "user".into() => json!({
                    "name": "John Doe",
                    "age": 25,
                })
            ]),
            url: inertia.url,
            version: Some(inertia.get_version()),
        };

        let view_data = ViewData {
            custom_props: Map::from_iter(hashmap!["ssr".into() => false.into()]),
            page,
            ssr_page: None,
        };

        let result = inertia.template_resolver.resolve_template(view_data).await;

        assert!(result.is_ok());

        let page = result.unwrap();

        assert_eq!(
            page.replace("    ", "").replace("\r\n", "\n").trim(),
            r#"<!doctype html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
                <meta http-equiv="X-UA-Compatible" content="ie=edge">
                <meta name="ssr" value="false">
                <title inertia>John Doe</title>
                <script type="module">
                            import RefreshRuntime from 'http://localhost:5173/@react-refresh'
                            RefreshRuntime.injectIntoGlobalHook(window)
                            window.$RefreshReg$ = () => {}
                            window.$RefreshSig$ = () => (type) => type
                            window.__vite_plugin_react_preamble_installed__ = true
                        </script>
                <script type="module" src="http://localhost:5173/www/app.tsx"></script>
            <script type="module" src="http://localhost:5173/@vite/client"></script>

            </head>
            <body>
            <div id='app' data-page='{"component":"Index","props":{"user":{"age":25,"name":"John Doe"}},"url":"http://localhost:3000","version":"1","clearHistory":false,"encryptHistory":false}'></div>

            </body>
            </html>
            "#.replace("    ", "").replace("\r\n", "\n").trim()
        );
    }

    #[tokio::test]
    async fn render_on_manifest_mode() {
        let vite = Vite::new(
            ViteConfig::new("tests/common/manifest.json", vec!["www/app.tsx"])
                .set_force_mode(ViteMode::Manifest),
        )
        .await
        .unwrap();

        let template_resolver =
            ViteHBSTemplateResolver::new(vite, "tests/common/root_layout.hbs", false).unwrap();

        let inertia = get_inertia(template_resolver).await;

        let page = InertiaPage {
            clear_history: false,
            encrypt_history: false,
            component: "Index".into(),
            deferred_props: None,
            merge_props: None,
            props: Map::from_iter(hashmap![
                "user".into() => json!({
                    "name": "John Doe",
                    "age": 25,
                })
            ]),
            url: inertia.url,
            version: Some(inertia.get_version()),
        };

        let view_data = ViewData {
            custom_props: Map::from_iter(hashmap!["ssr".into() => true.into()]),
            page,
            ssr_page: Some(InertiaSSRPage {
                body: "<h1>Hello John Doe!</h1><p>You are 25 years old.</p>".into(),
                head: vec![
                    "<title inertia>Hello World John Doe</title>".into(),
                    r#"<meta name="description" content="Pretty interesting page description""#
                        .into(),
                ],
            }),
        };

        let result = inertia.template_resolver.resolve_template(view_data).await;

        assert!(result.is_ok());

        let page = result.unwrap();

        assert_eq!(
            page.replace("    ", "").replace("\r\n", "\n").replace("\n\n", "\n").trim(),
            r#"<!doctype html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
                <meta http-equiv="X-UA-Compatible" content="ie=edge">
                <meta name="ssr" value="true">
                <title inertia>John Doe</title>
                <link rel="stylesheet" href="assets/app-BoNdGpDJ.css" />
                <script type="module" src="assets/app-CEfUlegz.js"></script>
                <title inertia>Hello World John Doe</title>
                <meta name="description" content="Pretty interesting page description"
            </head>
            <body>
                <h1>Hello John Doe!</h1><p>You are 25 years old.</p>
            </body>
            </html>"#.replace("    ", "").replace("\r\n", "\n").replace("\n\n", "\n").trim()
        )
    }
}
