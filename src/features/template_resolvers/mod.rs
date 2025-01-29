#[cfg(feature = "vite-template-resolver")]
mod vite_template_resolver;

#[cfg(feature = "vite-template-resolver")]
pub use vite_template_resolver::ViteTemplateResolver;

#[cfg(feature = "vite-hbs-template-resolver")]
mod vite_hbs_template_resolver;

#[cfg(feature = "vite-hbs-template-resolver")]
pub use vite_hbs_template_resolver::ViteHBSTemplateResolver;
