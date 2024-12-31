use crate::{
    page::DeferredProps,
    req_type::{InertiaRequestType, PartialComponent},
};
use serde_json::{Map, Value};
use std::{collections::HashMap, sync::Arc};

type PropResolver = Arc<dyn Fn() -> Value + Send + Sync>;

pub type InertiaProps<'a> = HashMap<&'a str, InertiaProp<'a>>;

#[derive(Clone)]
pub enum InertiaProp<'a> {
    /// - ALWAYS included on standard visits
    /// - OPTIONALLY included on partial reloads
    /// - ALWAYS evaluated
    Data(Value),
    /// - ALWAYS included on standard visits
    /// - OPTIONALLY included on partial reloads
    /// - ONLY evaluated when included
    Lazy(PropResolver),
    /// - ALWAYS included on standard visits
    /// - ALWAYS included on partial reloads (even if not requested or excepted)
    /// - ALWAYS evaluated
    Always(Value),
    /// - NEVER included on standard visits
    /// - OPTIONALLY included on partial reloads
    /// - ONLY evaluated when needed
    Demand(PropResolver),
    /// Exactly the same as `InertiaProp::Demand`, except that it will be automatically
    /// fetched by Inertia when the page is first loaded.
    ///
    /// Refer to [Deferred Props](https://inertiajs.com/deferred-props) for
    /// more details.
    Deferred(PropResolver, Option<&'a str>),
    /// Make a property mergeable. Refer to [Merging Props](https://inertiajs.com/merging-props)
    /// documentation for more details.
    ///
    /// It can only hold the `InertiaProp::Data` and `InertiaProp::Deferred` variants of `InertiaProp`.
    Mergeable(Box<InertiaProp<'a>>),
}

impl<'a> InertiaProp<'a> {
    #[inline]
    fn resolve_unconditionally(self) -> Value {
        match self {
            InertiaProp::Always(value) => value,
            InertiaProp::Data(value) => value,
            InertiaProp::Lazy(resolver) => resolver(),
            InertiaProp::Demand(resolver) => resolver(),
            InertiaProp::Deferred(resolver, _group) => resolver(),
            InertiaProp::Mergeable(prop) => prop.resolve_unconditionally(),
        }
    }

    /// Converts an `InertiaProp to `InertiaMergeableProp`.
    ///
    /// # Panics
    /// Will panic if the prop isn't neither `InertiaProp::Data` nor `InertiaProp::Deferred`
    /// variants.
    #[allow(dead_code)]
    fn into_mergeable(self) -> InertiaProp<'a> {
        match self {
            InertiaProp::Data(_) | InertiaProp::Deferred(_, _) => (),
            _ => panic!("You've tried to convert an invalid variant of InertiaProp into InertiaMergeableProp."),
        }

        InertiaProp::Mergeable(Box::new(self))
    }
}

#[inline]
pub(crate) fn resolve_props<'a>(
    raw_props: &'a InertiaProps<'a>,
    req_type: &InertiaRequestType,
) -> Map<String, Value> {
    let mut props = Map::new();

    match req_type {
        InertiaRequestType::Standard => {
            for (key, prop) in raw_props.iter() {
                if matches!(prop, InertiaProp::Demand(_) | InertiaProp::Deferred(_, _)) {
                    continue;
                }

                if let InertiaProp::Mergeable(prop) = prop {
                    if matches!(**prop, InertiaProp::Deferred(_, _)) {
                        continue;
                    }
                }

                props.insert(key.to_string(), prop.clone().resolve_unconditionally());
            }
        }

        InertiaRequestType::Partial(partial) => raw_props.iter().for_each(|(key, prop)| {
            let key = key.to_string();
            match prop {
                InertiaProp::Always(value) => {
                    props.insert(key, value.clone());
                }

                InertiaProp::Data(value) => {
                    if should_be_pushed(&key, partial) {
                        props.insert(key, value.clone());
                    }
                }

                InertiaProp::Lazy(resolver) => {
                    if should_be_pushed(&key, partial) {
                        props.insert(key, resolver());
                    }
                }

                InertiaProp::Demand(resolver) => {
                    if should_be_pushed(&key, partial) {
                        props.insert(key, resolver());
                    }
                }

                InertiaProp::Deferred(resolver, _) => {
                    if should_be_pushed(&key, partial) {
                        props.insert(key, resolver());
                    }
                }

                InertiaProp::Mergeable(prop) => match &**prop {
                    InertiaProp::Data(value) => {
                        if should_be_pushed(&key, partial) {
                            props.insert(key.to_string(), value.clone());
                        }
                    }

                    InertiaProp::Deferred(resolver, _) => {
                        if should_be_pushed(&key, partial) {
                            props.insert(key.to_string(), resolver());
                        }
                    }

                    _ => (),
                },
            };
        }),
    };

    props
}

#[inline]
fn should_be_pushed(key: &String, partial: &PartialComponent) -> bool {
    partial.only.contains(key) || partial.only.is_empty() && !partial.except.contains(key)
}

#[inline]
pub fn get_mergeable_props<'b>(
    props: &'b InertiaProps<'b>,
    keys_to_reset: Vec<&'b str>,
) -> Option<Vec<&'b str>> {
    let props = props
        .iter()
        .filter(|(key, prop)| {
            matches!(**prop, InertiaProp::Mergeable(_)) && !keys_to_reset.contains(*key)
        })
        .map(|(key, _)| *key)
        .collect::<Vec<_>>();

    match props.is_empty() {
        true => None,
        false => Some(props),
    }
}

#[inline]
pub fn get_deferred_props<'b>(
    props: &'b InertiaProps<'b>,
    req_type: &InertiaRequestType,
) -> DeferredProps<'b> {
    if req_type.is_partial() {
        return None;
    }

    let mut deferred_props = HashMap::new();

    for (key, prop) in props.iter() {
        let group;

        if let &InertiaProp::Deferred(_, _group) = prop {
            group = _group.unwrap_or("default");
        } else if let InertiaProp::Mergeable(prop) = prop {
            if let InertiaProp::Deferred(_, _group) = &**prop {
                group = _group.unwrap_or("default");
            } else {
                continue;
            }
        } else {
            continue;
        }

        if !deferred_props.contains_key(group) {
            deferred_props.insert(group, vec![*key]);
        } else {
            deferred_props.get_mut(group).unwrap().push(*key);
        }
    }

    match deferred_props.is_empty() {
        true => None,
        false => Some(deferred_props),
    }
}
