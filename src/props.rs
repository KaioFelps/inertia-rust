use crate::{
    error::IntoInertiaError,
    page::DeferredProps,
    req_type::{InertiaRequestType, PartialComponent},
    InertiaError,
};
use serde::Serialize;
use serde_json::{to_value, Map, Value};
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

type PropResolver = Arc<
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<Value, InertiaError>> + Send>> + Send + Sync,
>;

pub type InertiaProps<'a> = HashMap<&'a str, InertiaProp<'a>>;

pub trait IntoInertiaPropResult {
    /// Converts a serializeable object into a [`serde_json::Value`]. If it fails,
    /// automatically maps the error to [`InertiaError`].
    fn into_inertia_value(self) -> Result<Value, InertiaError>;
}

impl<T: Serialize> IntoInertiaPropResult for T {
    fn into_inertia_value(self) -> Result<Value, InertiaError> {
        to_value(self).map_err(IntoInertiaError::into_inertia_error)
    }
}

#[derive(Clone)]
pub enum InertiaProp<'a> {
    /// - ALWAYS included on standard visits
    /// - OPTIONALLY included on partial reloads
    /// - ALWAYS evaluated
    Data(Result<Value, InertiaError>),
    /// - ALWAYS included on standard visits
    /// - OPTIONALLY included on partial reloads
    /// - ONLY evaluated when included
    Lazy(PropResolver),
    /// - ALWAYS included on standard visits
    /// - ALWAYS included on partial reloads (even if not requested or excepted)
    /// - ALWAYS evaluated
    Always(Result<Value, InertiaError>),
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
    pub(crate) async fn resolve_unconditionally(self) -> Result<Value, InertiaError> {
        match self {
            InertiaProp::Always(value) => value,
            InertiaProp::Data(value) => value,
            InertiaProp::Demand(resolver) => resolver()
                .await
                .map_err(IntoInertiaError::into_inertia_error),
            InertiaProp::Deferred(resolver, _group) => resolver()
                .await
                .map_err(IntoInertiaError::into_inertia_error),
            InertiaProp::Mergeable(prop) => Box::pin(prop.resolve_unconditionally())
                .await
                .map_err(IntoInertiaError::into_inertia_error),
            InertiaProp::Lazy(resolver) => resolver()
                .await
                .map_err(IntoInertiaError::into_inertia_error),
        }
    }

    /// Converts an `InertiaProp to `InertiaMergeableProp`.
    ///
    /// # Panics
    /// Will panic if the prop isn't neither `InertiaProp::Data` nor `InertiaProp::Deferred`
    /// variants.
    #[allow(dead_code)]
    pub fn into_mergeable(self) -> InertiaProp<'a> {
        match self {
            InertiaProp::Data(_) | InertiaProp::Deferred(_, _) => (),
            _ => panic!("You've tried to convert an invalid variant of InertiaProp into InertiaMergeableProp."),
        }

        InertiaProp::Mergeable(Box::new(self))
    }

    pub fn data<T>(value: T) -> InertiaProp<'a>
    where
        T: Serialize,
    {
        InertiaProp::Data(value.into_inertia_value())
    }

    pub fn always<T>(value: T) -> InertiaProp<'a>
    where
        T: Serialize,
    {
        InertiaProp::Always(value.into_inertia_value())
    }

    pub fn merge<T>(value: T) -> InertiaProp<'a>
    where
        T: Serialize,
    {
        let prop = InertiaProp::Data(value.into_inertia_value());
        InertiaProp::Mergeable(Box::new(prop))
    }

    pub fn lazy(resolver: PropResolver) -> InertiaProp<'a> {
        InertiaProp::Lazy(resolver)
    }

    pub fn demand(resolver: PropResolver) -> InertiaProp<'a> {
        InertiaProp::Demand(resolver)
    }

    pub fn defer(resolver: PropResolver) -> InertiaProp<'a> {
        InertiaProp::Deferred(resolver, None)
    }

    pub fn defer_with_group(resolver: PropResolver, group: &'a str) -> InertiaProp<'a> {
        InertiaProp::Deferred(resolver, Some(group))
    }
}

#[inline]
pub(crate) async fn resolve_props<'a>(
    raw_props: &'a InertiaProps<'a>,
    req_type: &InertiaRequestType,
) -> Result<Map<String, Value>, InertiaError> {
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

                match prop.clone().resolve_unconditionally().await {
                    Ok(value) => {
                        props.insert(key.to_string(), value);
                    }

                    Err(err) => {
                        log::error!("Failed to resolve prop {}: {}", key, err);
                    }
                }
            }
        }

        InertiaRequestType::Partial(partial) => {
            for (key, prop) in raw_props {
                let key = key.to_string();

                match prop {
                    InertiaProp::Always(value) => {
                        let value = value.clone().map_err(|err| {
                            log::error!("Failed to resolve always prop {}: {}", &key, err);
                            err
                        })?;

                        if should_be_pushed(&key, partial) {
                            props.insert(key, value);
                        }
                    }

                    InertiaProp::Data(value) => {
                        let value = value.clone().map_err(|err| {
                            log::error!("Failed to resolve data prop {}: {}", &key, err);
                            err
                        })?;

                        if should_be_pushed(&key, partial) {
                            props.insert(key, value);
                        }
                    }

                    InertiaProp::Lazy(resolver) => {
                        let value = resolver().await.map_err(|err| {
                            log::error!("Failed to resolve lazy prop {}: {}", &key, err);
                            err
                        })?;

                        if should_be_pushed(&key, partial) {
                            props.insert(key, value);
                        }
                    }

                    InertiaProp::Demand(resolver) => {
                        let value = resolver().await.map_err(|err| {
                            log::error!("Failed to resolve demand prop {}: {}", &key, err);
                            err
                        })?;

                        if should_be_pushed(&key, partial) {
                            props.insert(key, value);
                        }
                    }

                    InertiaProp::Deferred(resolver, _) => {
                        let value = resolver().await.map_err(|err| {
                            log::error!("Failed to resolve deferred prop {}: {}", &key, err);
                            err
                        })?;

                        if should_be_pushed(&key, partial) {
                            props.insert(key, value);
                        }
                    }

                    InertiaProp::Mergeable(prop) => match &**prop {
                        InertiaProp::Data(value) => {
                            let value = value.clone().map_err(|err| {
                                log::error!(
                                    "Failed to resolve mergeable data prop {}: {}",
                                    &key,
                                    err
                                );
                                err
                            })?;

                            if should_be_pushed(&key, partial) {
                                props.insert(key, value);
                            }
                        }

                        InertiaProp::Deferred(resolver, _) => {
                            let value = resolver().await.map_err(|err| {
                                log::error!(
                                    "Failed to resolve mergeable deferred prop {}: {}",
                                    &key,
                                    err
                                );
                                err
                            })?;

                            if should_be_pushed(&key, partial) {
                                props.insert(key, value);
                            }
                        }

                        _ => (),
                    },
                };
            }
        }
    };

    Ok(props)
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

#[cfg(test)]
mod test {
    use crate::props::{get_deferred_props, get_mergeable_props, InertiaProp};
    use crate::req_type::{InertiaRequestType, PartialComponent};
    use crate::{hashmap, prop_resolver, Component, InertiaPage};
    use actix_web::test;
    use serde::Serialize;
    use serde_json::{json, to_value, Value};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use super::resolve_props;

    #[test]
    async fn test_inertia_partials_visit_page() {
        #[derive(Serialize)]
        struct Events {
            id: u16,
            title: String,
        }

        let event = Events {
            id: 1,
            title: "Baile".into(),
        };

        let props = hashmap![
            "event" => InertiaProp::Data(json!({"name": "John Doe"})),
            "categories" => InertiaProp::Data(vec!["foo".to_string(), "bar".to_string()].into()),
            "events" => InertiaProp::Data(
                to_value(vec![to_value(event).unwrap()]).unwrap(),
            )
        ];

        // Request headers
        // X-Inertia: true
        // X-Inertia-Version: generated_version
        // X-Inertia-Partial-Data: events
        // X-Inertia-Partial-Component: Events
        let req_type = InertiaRequestType::Partial(PartialComponent {
            component: Component("Events".to_string()),
            only: Vec::from(["events".to_string()]),
            except: Vec::new(),
        });

        let page = InertiaPage::new(
            Component("Events".into()),
            "/events/80",
            Some("generated_version"),
            resolve_props(&props, &req_type).await,
            None,
            None,
            false,
            false,
        );

        let json_page_example = json!({
            "clearHistory":false,
            "component": "Events",
            "encryptHistory":false,
            "props": {
            // "auth": { "name": "John Doe" },              // NOT included
            // "categories": ["foo", "bar"],                // NOT included
            "events": [{"id": 1, "title": "Baile"}]      // included
            },
            "url": "/events/80",
            "version": "generated_version",

        });

        assert_eq!(
            json!(page).to_string(),
            serde_json::to_string(&json_page_example).unwrap(),
        );
    }

    #[test]
    async fn test_inertia_standard_visit_page() {
        let props = hashmap! [
            "radioStatus" => InertiaProp::Demand(Arc::new(|| Box::pin(async move { json!({"announcer": "John Doe"}) }))),
            "categories" => InertiaProp::Data(vec!["foo".to_string(), "bar".to_string()].into())
        ];

        // Request headers
        // X-Inertia: true
        // X-Inertia-Version: generated_version
        let req_type = InertiaRequestType::Standard;

        let page = InertiaPage::new(
            Component("Categories".into()),
            "/categories",
            Some("generated_version"),
            resolve_props(&props, &req_type).await,
            None,
            None,
            false,
            false,
        );

        let json_page_example = json!({
            "clearHistory": false,
            "component": "Categories",
            "encryptHistory": false,
            "props": {
            // "radioStatus": { "announcer": "John Doe" },  // NOT included
            "categories": ["foo", "bar"],                   // included
            },
            "url": "/categories",
            "version": "generated_version"
        });

        assert_eq!(
            json!(page).to_string(),
            serde_json::to_string(&json_page_example).unwrap(),
        );
    }

    fn get_deferred_props_hashmap<'a>() -> HashMap<&'a str, InertiaProp<'a>> {
        hashmap![
            "users" => InertiaProp::Deferred(prop_resolver!({ to_value(vec!["user1", "user2", "user3"]).unwrap() }), Some("users")),
            "permissions" => InertiaProp::Deferred(prop_resolver!({ to_value(vec!["delete", "update", "read"]).unwrap()}), Some("users")),
            "events" => InertiaProp::Deferred(prop_resolver!({ to_value(vec!["event1", "event2", "event3"]).unwrap() }), None)
        ]
    }

    #[test]
    async fn test_standard_request_deferred_props_behavior() {
        let props = get_deferred_props_hashmap();

        let standard_page = json!(InertiaPage {
            deferred_props: get_deferred_props(&props, &InertiaRequestType::Standard),
            component: "Foo".into(),
            clear_history: false,
            encrypt_history: false,
            merge_props: None,
            props: resolve_props(&props, &InertiaRequestType::Standard).await,
            url: "foo",
            version: Some("foo")
        });

        assert!(
            standard_page.clone()["deferredProps"]["default"]
                .as_array()
                .unwrap()
                .contains(&to_value("events").unwrap()),
            "Deferred Props field from standard visit should contain an 'default' gorup containing 'events' key."
        );

        assert!([
            to_value("users").unwrap(),
            to_value("permissions").unwrap()
        ]
        .iter()
        .all(|key| standard_page.clone()["deferredProps"]["users"]
            .as_array()
            .unwrap()
            .contains(key)),
            "Deferred Props field from standard visit should contain an 'user' group containing 'users' and 'permissions' keys."
        );

        assert!(standard_page["props"].as_object().unwrap().is_empty(), "Props field should be empty once there is only deferred props in it and it's an standard request.");
    }

    #[test]
    async fn test_partial_request_for_default_group_from_deferred_props_behavior() {
        let props = get_deferred_props_hashmap();

        // partial request for 'default' group only contains 'events' prop
        let partial_req_for_default = InertiaRequestType::Partial(PartialComponent {
            component: "Foo".into(),
            only: vec!["events".into()],
            except: vec![],
        });

        let default_partial_page = json!(InertiaPage {
            deferred_props: get_deferred_props(&props, &partial_req_for_default),
            component: "Foo".into(),
            clear_history: false,
            encrypt_history: false,
            merge_props: None,
            props: resolve_props(&props, &partial_req_for_default).await,
            url: "foo",
            version: Some("foo")
        });

        assert!(
            default_partial_page.get("deferredProps").is_none(),
            "'deferredProps' field should not exist in partial requests."
        );

        assert!(
            default_partial_page["props"]
                .as_object()
                .unwrap()
                .get("events")
                .is_some_and(
                    |events| ["event1", "event2", "event3"].iter().all(|event| events
                        .as_array()
                        .unwrap()
                        .contains(&Value::String(event.to_string())))
                ),
            "partial request for 'default' group should contain 'events' list in 'props' field with the props events values."
        )
    }

    #[test]
    async fn test_partial_request_for_users_group_from_deferred_props_behavior() {
        let props = get_deferred_props_hashmap();

        let partial_req_for_users = InertiaRequestType::Partial(PartialComponent {
            component: "Foo".into(),
            only: vec!["users".into(), "permissions".into()],
            except: vec![],
        });

        let users_partial_page = json!(InertiaPage {
            deferred_props: get_deferred_props(&props, &partial_req_for_users),
            component: "Foo".into(),
            clear_history: false,
            encrypt_history: false,
            merge_props: None,
            props: resolve_props(&props, &partial_req_for_users).await,
            url: "foo",
            version: Some("foo")
        });

        assert!(
            users_partial_page.get("deferredProps").is_none(),
            "'deferredProps' field should not exist in partial requests."
        );

        assert!(users_partial_page["props"]
            .as_object()
            .unwrap()
            .get("users")
            .is_some_and(
                |users| ["user1", "user2", "user3"].iter().all(|user| users
                    .as_array()
                    .unwrap()
                    .contains(&to_value(user).unwrap()))
            ),
            "'props' field should contain an 'users' group which should be a list containing the values from given props hashmap 'users' field."
        );

        assert!(users_partial_page["props"]
            .as_object()
            .unwrap()
            .get("permissions")
            .is_some_and(
                |permissions| ["delete", "update", "read"].iter().all(|permission| permissions
                    .as_array()
                    .unwrap()
                    .contains(&to_value(permission).unwrap()))
            ),
            "'props' field should contain an 'permissions' group which should be a list containing the values from given props hashmap 'permissions' field."
        );
    }

    #[test]
    async fn test_mergeable_props_behavior_without_reset_list() {
        let get_inertia_pages = move |page: usize| {
            let users_memory_db = Arc::new(vec!["user1", "user2", "user3", "user4", "user5"]);
            let permissions_memory_db = ["read", "update", "delete"];

            let props = hashmap![
                "permissions" => InertiaProp::Mergeable(Box::new(InertiaProp::Data(to_value(
                    permissions_memory_db.iter().skip((page -1) * 2).take(2).cloned().collect::<Vec<_>>()
                ).unwrap()))),
                "users" => InertiaProp::defer(prop_resolver!(
                    let users = users_memory_db.clone();
                    {
                    to_value(users
                        .clone()
                        .iter()
                        .skip((page - 1) * 3)
                        .take(3)
                        .cloned()
                        .collect::<Vec<_>>())
                        .unwrap()
                    }))
                    .into_mergeable()
            ];

            let partial_req = InertiaRequestType::Partial(PartialComponent {
                component: "Foo".into(),
                except: vec![],
                only: vec!["users".into()],
            });

            async move {
                (
                    json!(InertiaPage {
                        clear_history: false,
                        encrypt_history: false,
                        component: "Foo".into(),
                        deferred_props: get_deferred_props(&props, &InertiaRequestType::Standard),
                        merge_props: get_mergeable_props(&props, vec![]),
                        props: resolve_props(&props, &InertiaRequestType::Standard).await,
                        url: "",
                        version: Some("")
                    }),
                    json!(InertiaPage {
                        clear_history: false,
                        encrypt_history: false,
                        component: "Foo".into(),
                        deferred_props: get_deferred_props(&props, &partial_req),
                        merge_props: get_mergeable_props(&props, vec![]),
                        props: resolve_props(&props, &partial_req).await,
                        url: "",
                        version: Some("")
                    }),
                )
            }
        };

        let page = Arc::new(Mutex::new(1));
        let _page = *page.lock().unwrap();
        let (standard_page, partial_page) = get_inertia_pages(_page).await;

        assert!(standard_page["props"]
            .as_object()
            .unwrap()
            .contains_key("permissions"));

        assert!(partial_page["props"]
            .as_object()
            .unwrap()
            .contains_key("users"));

        assert!(["permissions", "users"]
            .iter()
            .all(|prop| standard_page["mergeProps"]
                .as_array()
                .is_some_and(|props| props.contains(&to_value(prop).unwrap()))));

        assert!(standard_page["deferredProps"]["default"]
            .as_array()
            .unwrap()
            .contains(&to_value("users").unwrap()));

        assert!(["user1", "user2", "user3"]
            .iter()
            .all(|user| partial_page["props"]["users"]
                .as_array()
                .is_some_and(|users| users.contains(&to_value(user).unwrap()))));

        assert!(["read", "update"]
            .iter()
            .all(|permission| standard_page["props"]["permissions"]
                .as_array()
                .is_some_and(|permissions| permissions.contains(&to_value(permission).unwrap()))));

        //
        // second page
        //
        *page.lock().unwrap() = 2;
        let _page = *page.lock().unwrap();
        let (standard_page, partial_page) = get_inertia_pages(_page).await;

        log::info!("{}\n\n", standard_page);
        log::info!("{}\n\n", partial_page);

        assert!(partial_page["props"]
            .as_object()
            .unwrap()
            .contains_key("users"));

        assert!(standard_page["props"]
            .as_object()
            .unwrap()
            .contains_key("permissions"));

        assert!(["permissions", "users"]
            .iter()
            .all(|prop| standard_page["mergeProps"]
                .as_array()
                .is_some_and(|props| props.contains(&to_value(prop).unwrap()))));

        assert!(standard_page["deferredProps"]["default"]
            .as_array()
            .unwrap()
            .contains(&to_value("users").unwrap()));

        assert!(["user4", "user5"]
            .iter()
            .all(|user| partial_page["props"]["users"]
                .as_array()
                .is_some_and(|users| users.contains(&to_value(user).unwrap()))));

        assert!(standard_page["props"]["permissions"]
            .as_array()
            .is_some_and(|permissions| permissions.eq(&["delete"])));
    }

    #[test]
    async fn test_mergeable_props_behavior_with_reset() {
        async fn get_inertia_page(page: usize, keys_to_reset: &[&str]) -> Value {
            let permissions_mem_db = ["read", "update", "delete"];
            let per_page = 2;

            let props = hashmap![
                "permissions" => InertiaProp::Data(to_value(
                    permissions_mem_db
                    .iter()
                    .skip((page -1) * per_page)
                    .take(per_page)
                    .cloned()
                    .collect::<Vec<_>>())
                    .unwrap())
                    .into_mergeable()
            ];

            json!(InertiaPage {
                clear_history: false,
                encrypt_history: false,
                component: "Foo".into(),
                deferred_props: None,
                merge_props: get_mergeable_props(&props, keys_to_reset.to_vec()),
                props: resolve_props(&props, &InertiaRequestType::Standard).await,
                url: "",
                version: None,
            })
        }

        let page = Arc::new(Mutex::new(1));
        let _page = *page.lock().unwrap();
        let inertia_page = get_inertia_page(_page, &[]).await;

        assert!(inertia_page["mergeProps"]
            .as_array()
            .unwrap()
            .contains(&to_value("permissions").unwrap()));

        assert!(["read", "update"]
            .iter()
            .all(|permission| inertia_page["props"]["permissions"]
                .as_array()
                .unwrap()
                .contains(&to_value(permission).unwrap())));

        *page.lock().unwrap() = 2;
        let _page = *page.lock().unwrap();
        let inertia_page = get_inertia_page(_page, &["permissions"]).await;

        assert!(inertia_page.get("mergeProps").is_none());

        assert!(inertia_page["props"]["permissions"]
            .as_array()
            .unwrap()
            .eq(&["delete"]));
    }
}
