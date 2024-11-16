#[cfg(feature = "basic-vite-resolver")]
mod basic_vite_resolver;

#[cfg(feature = "basic-vite-resolver")]
pub use basic_vite_resolver::template_resolver as basic_vite_resolver;
