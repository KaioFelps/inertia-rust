use crate::req_type::{InertiaRequestType, PartialComponent};
use serde_json::{Map, Value};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub enum InertiaProp {
    /// - ALWAYS included on standard visits
    /// - OPTIONALLY included on partial reloads
    /// - ALWAYS evaluated
    Data(Value),
    /// - ALWAYS included on standard visits
    /// - OPTIONALLY included on partial reloads
    /// - ONLY evaluated when included
    Lazy(Arc<dyn Fn() -> Value + Send + Sync>),
    /// - ALWAYS included on standard visits
    /// - ALWAYS included on partial reloads (even if not requested or excepted)
    /// - ALWAYS evaluated
    Always(Value),
    /// - NEVER included on standard visits
    /// - OPTIONALLY included on partial reloads
    /// - ONLY evaluated when needed
    Demand(Arc<dyn Fn() -> Value + Send + Sync>),
}

impl InertiaProp {
    #[inline]
    pub(crate) fn resolve_props(
        raw_props: &InertiaProps,
        req_type: InertiaRequestType,
    ) -> Map<String, Value> {
        let mut props = Map::new();

        if req_type.is_standard() {
            for (key, value) in raw_props.iter() {
                if let InertiaProp::Demand(_) = value {
                    continue;
                }

                props.insert(key.clone(), value.clone().resolve_prop_unconditionally());
            }

            return props;
        }

        let partials = req_type.unwrap_partial();

        for (key, value) in raw_props.iter() {
            match value {
                InertiaProp::Always(value) => {
                    props.insert(key.clone(), value.clone());
                }
                InertiaProp::Data(value) => {
                    if Self::should_be_pushed(key, &partials) {
                        props.insert(key.clone(), value.clone());
                    }
                }
                InertiaProp::Lazy(resolver) => {
                    if Self::should_be_pushed(key, &partials) {
                        props.insert(key.clone(), resolver());
                    }
                }
                InertiaProp::Demand(resolver) => {
                    if Self::should_be_pushed(key, &partials) {
                        props.insert(key.clone(), resolver());
                    }
                }
            };
        }

        props
    }

    #[inline]
    fn resolve_prop_unconditionally(self) -> Value {
        match self {
            InertiaProp::Always(value) => value,
            InertiaProp::Data(value) => value,
            InertiaProp::Demand(resolver) => resolver(),
            InertiaProp::Lazy(resolver) => resolver(),
        }
    }

    #[inline]
    fn should_be_pushed(key: &String, partial: &PartialComponent) -> bool {
        partial.only.contains(key) || partial.only.is_empty() && !partial.except.contains(key)
    }
}

pub type InertiaProps = HashMap<String, InertiaProp>;
