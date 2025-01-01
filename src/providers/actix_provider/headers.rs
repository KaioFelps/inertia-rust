use crate::{inertia, Component};
use actix_web::http::header::{HeaderName, HeaderValue};

pub const X_INERTIA: HeaderName = HeaderName::from_static(inertia::X_INERTIA);
pub const X_INERTIA_LOCATION: HeaderName = HeaderName::from_static(inertia::X_INERTIA_LOCATION);
pub const X_INERTIA_VERSION: HeaderName = HeaderName::from_static(inertia::X_INERTIA_VERSION);
pub const X_INERTIA_PARTIAL_COMPONENT: HeaderName =
    HeaderName::from_static(inertia::X_INERTIA_PARTIAL_COMPONENT);

pub const X_INERTIA_PARTIAL_DATA: HeaderName =
    HeaderName::from_static(inertia::X_INERTIA_PARTIAL_DATA);

pub const X_INERTIA_PARTIAL_EXCEPT: HeaderName =
    HeaderName::from_static(inertia::X_INERTIA_PARTIAL_EXCEPT);

pub const X_INERTIA_RESET: HeaderName = HeaderName::from_static(inertia::X_INERTIA_RESET);

pub const X_INERTIA_ERROR_BAG: HeaderName = HeaderName::from_static(inertia::X_INERTIA_ERROR_BAG);

pub enum InertiaHeader<'a> {
    Inertia,
    InertiaLocation(&'a str),
    InertiaPartialData(Vec<&'a str>),
    InertiaPartialExcept(Vec<&'a str>),
    InertiaPartialComponent(Component),
    InertiaReset(Vec<&'a str>),
    InertiaErrorBag(&'a str),
    Version(&'a str),
}

impl InertiaHeader<'_> {
    pub fn convert(&self) -> (HeaderName, HeaderValue) {
        match self {
            Self::Inertia => (X_INERTIA, HeaderValue::from_str("true").unwrap()),
            Self::Version(version) => (X_INERTIA_VERSION, HeaderValue::from_str(version).unwrap()),
            Self::InertiaLocation(path) => {
                (X_INERTIA_LOCATION, HeaderValue::from_str(path).unwrap())
            }
            Self::InertiaPartialData(partials) => (
                X_INERTIA_PARTIAL_DATA,
                HeaderValue::from_str(&partials.join(",")).unwrap(),
            ),
            Self::InertiaPartialExcept(partials) => (
                X_INERTIA_PARTIAL_EXCEPT,
                HeaderValue::from_str(&partials.join(",")).unwrap(),
            ),
            Self::InertiaReset(reset) => (
                X_INERTIA_PARTIAL_EXCEPT,
                HeaderValue::from_str(&reset.join(",")).unwrap(),
            ),
            Self::InertiaPartialComponent(Component(component)) => (
                X_INERTIA_PARTIAL_COMPONENT,
                HeaderValue::from_str(component).unwrap(),
            ),
            Self::InertiaErrorBag(bag) => {
                (X_INERTIA_ERROR_BAG, HeaderValue::from_str(bag).unwrap())
            }
        }
    }
}
