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

#[cfg(test)]
mod test {
    use crate::props::{get_deferred_props, get_mergeable_props, resolve_props, InertiaProp};
    use crate::req_type::{InertiaRequestType, PartialComponent};
    use crate::{hashmap, Component, InertiaPage};
    use actix_web::test;
    use serde::Serialize;
    use serde_json::{json, to_value, Value};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

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
                serde_json::to_value(vec![serde_json::to_value(event).unwrap()]).unwrap(),
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
            resolve_props(&props, &req_type),
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
            "radioStatus" => InertiaProp::Demand(Arc::new(|| json!({"announcer": "John Doe"}))),
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
            resolve_props(&props, &req_type),
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
            "users" => InertiaProp::Deferred(Arc::new(|| vec!["user1", "user2", "user3"].into()), Some("users")),
            "permissions" => InertiaProp::Deferred(Arc::new(|| vec!["delete", "update", "read"].into()), Some("users")),
            "events" => InertiaProp::Deferred(Arc::new(|| vec!["event1", "event2", "event3"].into()), None)
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
            props: resolve_props(&props, &InertiaRequestType::Standard),
            url: "foo",
            version: Some("foo")
        });

        assert!(
            standard_page.clone()["deferredProps"]["default"]
                .as_array()
                .unwrap()
                .contains(&serde_json::to_value("events").unwrap()),
            "Deferred Props field from standard visit should contain an 'default' gorup containing 'events' key."
        );

        assert!([
            serde_json::to_value("users").unwrap(),
            serde_json::to_value("permissions").unwrap()
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
            props: resolve_props(&props, &partial_req_for_default),
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
            props: resolve_props(&props, &partial_req_for_users),
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
                    .contains(&serde_json::to_value(user).unwrap()))
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
                    .contains(&serde_json::to_value(permission).unwrap()))
            ),
            "'props' field should contain an 'permissions' group which should be a list containing the values from given props hashmap 'permissions' field."
        );
    }

    #[test]
    async fn test_mergeable_props_behavior_without_reset_list() {
        let get_inertia_pages = |page: usize| -> (Value, Value) {
            let users_memory_db = Arc::new(vec!["user1", "user2", "user3", "user4", "user5"]);
            let permissions_memory_db = ["read", "update", "delete"];

            let props = hashmap![
                "permissions" => InertiaProp::Mergeable(Box::new(InertiaProp::Data(to_value(
                    permissions_memory_db.iter().skip((page -1) * 2).take(2).cloned().collect::<Vec<_>>()
                ).unwrap()))),
                "users" => InertiaProp::Deferred(Arc::new(move || to_value(users_memory_db
                    .iter()
                    .skip((page - 1) * 3)
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()).unwrap()), None)
                    .into_mergeable()
            ];

            let partial_req = InertiaRequestType::Partial(PartialComponent {
                component: "Foo".into(),
                except: vec![],
                only: vec!["users".into()],
            });

            (
                json!(InertiaPage {
                    clear_history: false,
                    encrypt_history: false,
                    component: "Foo".into(),
                    deferred_props: get_deferred_props(&props, &InertiaRequestType::Standard),
                    merge_props: get_mergeable_props(&props, vec![]),
                    props: resolve_props(&props, &InertiaRequestType::Standard),
                    url: "",
                    version: Some("")
                }),
                json!(InertiaPage {
                    clear_history: false,
                    encrypt_history: false,
                    component: "Foo".into(),
                    deferred_props: get_deferred_props(&props, &partial_req),
                    merge_props: get_mergeable_props(&props, vec![]),
                    props: resolve_props(&props, &partial_req,),
                    url: "",
                    version: Some("")
                }),
            )
        };

        let page = Arc::new(Mutex::new(1));
        let (standard_page, partial_page) = get_inertia_pages(*page.lock().unwrap() as usize);

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
        let (standard_page, partial_page) = get_inertia_pages(*page.lock().unwrap() as usize);

        println!("{}\n\n", standard_page);
        println!("{}\n\n", partial_page);

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
        let get_inertia_page = |page: usize, keys_to_reset: &[&str]| -> Value {
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
                props: resolve_props(&props, &InertiaRequestType::Standard),
                url: "",
                version: None,
            })
        };

        let page = Arc::new(Mutex::new(1));

        let inertia_page = get_inertia_page(*page.lock().unwrap() as usize, &[]);

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
        let inertia_page = get_inertia_page(*page.lock().unwrap() as usize, &["permissions"]);

        assert!(inertia_page.get("mergeProps").is_none());

        assert!(inertia_page["props"]["permissions"]
            .as_array()
            .unwrap()
            .eq(&["delete"]));
    }
}
