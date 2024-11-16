use crate::inertia;
use actix_web::http::header::{HeaderName, HeaderValue};

#[allow(unused)]
pub const X_INERTIA: HeaderName = HeaderName::from_static(inertia::X_INERTIA);
#[allow(unused)]
pub const X_INERTIA_LOCATION: HeaderName = HeaderName::from_static(inertia::X_INERTIA_LOCATION);
#[allow(unused)]
pub const X_INERTIA_VERSION: HeaderName = HeaderName::from_static(inertia::X_INERTIA_VERSION);
#[allow(unused)]
pub const X_INERTIA_PARTIAL_COMPONENT: HeaderName =
    HeaderName::from_static(inertia::X_INERTIA_PARTIAL_COMPONENT);
#[allow(unused)]
pub const X_INERTIA_PARTIAL_DATA: HeaderName =
    HeaderName::from_static(inertia::X_INERTIA_PARTIAL_DATA);
#[allow(unused)]
pub const X_INERTIA_PARTIAL_EXCEPT: HeaderName =
    HeaderName::from_static(inertia::X_INERTIA_PARTIAL_EXCEPT);

pub enum InertiaHeader<'a> {
    Inertia,
    InertiaLocation(&'a str),
    InertiaPartialData(Vec<&'a str>),
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
            Self::InertiaPartialData(partials) => {
                if partials.is_empty() {
                    return (X_INERTIA_PARTIAL_DATA, HeaderValue::from_str("").unwrap());
                }

                let mut str_partials = String::from(partials[0]);

                for part in partials[1..].iter() {
                    str_partials.push(',');
                    str_partials.push_str(part);
                }

                (
                    X_INERTIA_PARTIAL_DATA,
                    HeaderValue::from_str(str_partials.as_str()).unwrap(),
                )
            }
        }
    }
}
