#[macro_use] extern crate serde;

mod http_method;
mod inertia;
mod utils;
mod page;
mod error;
mod req_type;
mod props;

pub mod providers;

pub use inertia::Inertia as Inertia;
pub use inertia::Component;
pub use inertia::InertiaVersion;
pub use error::InertiaError;
pub use page::InertiaPage;
pub use page::InertiaSSRPage;
pub use props::InertiaProps;

#[cfg(test)]
mod tests {}
