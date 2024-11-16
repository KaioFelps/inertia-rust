use crate::Component;

#[derive(Eq, PartialEq, Debug)]
pub(crate) struct PartialComponent {
    pub component: Component,
    pub only: Vec<String>,
    pub except: Vec<String>,
}

pub(crate) enum InertiaRequestType {
    Standard,
    Partial(PartialComponent)
}

impl InertiaRequestType {
    #[inline]
    #[allow(unused)]
    pub fn is_standard(&self) -> bool { matches!(*self, InertiaRequestType::Standard) }

    #[inline]
    #[allow(unused)]
    pub fn is_partial(&self) -> bool { !self.is_standard() }

    #[inline]
    pub fn unwrap_partial(self) -> PartialComponent {
        match self {
            InertiaRequestType::Standard => {
                panic!("called `InertiaRequestType::unwrap_partial()` on an `Standard` request type value.");
            },
            InertiaRequestType::Partial(reqs) => reqs,
        }
    }

    #[inline]
    #[allow(unused)]
    pub fn partials(&self) -> Option<&PartialComponent> {
        match self {
            InertiaRequestType::Partial(reqs) => Some(reqs),
            InertiaRequestType::Standard => None,
        }
    }
}
