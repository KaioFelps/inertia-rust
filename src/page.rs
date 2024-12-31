use std::collections::HashMap;

use crate::inertia::Component;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Inertia Full Page response to be rendered inside the root template
/// on the first request or on a full visit request.
#[derive(Deserialize)]
pub struct InertiaSSRPage {
    /// All html-string elements to be injected in inertia_head, at the root template.
    pub(crate) head: Vec<String>,
    /// All html-string elements to be injected in inertia_body div container, at the root template.
    pub(crate) body: String,
}

impl InertiaSSRPage {
    /// Instantiates a new InertiaSSRPage object. See [`InertiaSSRPage`] struct docs for more
    /// details of its usage.
    ///
    /// [`InertiaSSRPage`]: InertiaSSRPage
    ///
    /// # Arguments
    /// * `head` -  A stringified html of the content to be injected in the layout
    ///             (given by [template_path]) head element (by innerHTML method).
    /// * `body` -  A stringified html of the body to be injected in the Inertia's div container
    ///             in the layout.
    ///
    /// [template_path]: crate::inertia::Inertia
    ///
    pub fn new(head: Vec<String>, body: String) -> Self {
        InertiaSSRPage { head, body }
    }

    pub fn get_head(&self) -> String {
        self.head.join("\n")
    }
    pub fn get_body(&self) -> &String {
        &self.body
    }
}

pub type DeferredProps<'a> = Option<HashMap<&'a str, Vec<&'a str>>>;

/// Response containing a valid Inertia Payload that will be used
/// by the Inertia client to render the components.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct InertiaPage<'a> {
    // The name of the JavaScript page component.
    pub(crate) component: Component,

    // A merge of page props and shared props .
    pub(crate) props: Map<String, Value>,

    // this is not the same as Inertia::url, that represents the application url.
    // this url represents the current request's url, i.e. the page url.
    pub(crate) url: &'a str,

    /// Current assets version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) version: Option<&'a str>,

    #[serde(rename = "clearHistory")]
    pub(crate) clear_history: bool,

    #[serde(rename = "encryptHistory")]
    pub(crate) encrypt_history: bool,

    #[serde(rename = "deferredProps")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) deferred_props: DeferredProps<'a>,

    #[serde(rename = "mergeProps")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) merge_props: Option<Vec<&'a str>>,
}

#[allow(clippy::too_many_arguments)]
impl<'a> InertiaPage<'a> {
    /// Instantiates an Inertia Page object to sent as http response,
    /// according to [Inertia Protocol].
    ///
    ///
    /// # Arguments
    /// * `component`       -   The name of the javascript page component (e.g. "/Me").
    /// * `url`             -   The Inertia instance's url (the application URL). It can be a
    ///                         whole href or an absolute hostless path ("/me").
    /// * `version`         -   Current assets version. Used to assert assets are up-to-date. See
    ///                         [Inertia's assets versioning] page for more details.
    /// * `props`           -   A map of the page's props.
    /// * `merge_props`     -   A list containing the keys of the properties that shall me merged by the
    ///                         client-side adapter.
    /// * `deferred_props`  -   A hashmap of which the keys are groups. It contains the keys of the props
    ///                         that must be fetched by the client-side adapter after the first page load.
    ///                         Refer to [Deferred Props] for more details.
    /// * `clear_history`   -   Whether the history must be cleaned by the client-side once this response is
    ///                         received. Refer to [Clearing history] section from History Encryption documentatin
    ///                         for more details.
    /// * `encrypt_history` -   Whether the client-side adapter must encrypt the history page data related to
    ///                         this response. Refer to [History Encryption] for more details.
    ///
    /// [Inertia Protocol]: https://inertiajs.com/the-protocol
    /// [Inertia's assets versioning]: https://inertiajs.com/the-protocol#asset-versioning
    /// [Deferred Props]: https://inertiajs.com/deferred-props
    /// [History Encryption]: https://inertiajs.com/history-encryption
    /// [Clearing history]: https://inertiajs.com/history-encryption#clearing-history
    pub fn new(
        component: Component,
        url: &'a str,
        version: Option<&'a str>,
        props: Map<String, Value>,
        merge_props: Option<Vec<&'a str>>,
        deferred_props: DeferredProps<'a>,
        clear_history: bool,
        encrypt_history: bool,
    ) -> Self {
        InertiaPage {
            component,
            url,
            props,
            version,
            merge_props,
            deferred_props,
            clear_history,
            encrypt_history,
        }
    }

    pub fn get_props(&self) -> &Map<String, Value> {
        &self.props
    }

    pub fn get_component(&self) -> &Component {
        &self.component
    }

    pub fn get_url(&self) -> &str {
        self.url
    }

    pub fn get_version(&self) -> &Option<&str> {
        &self.version
    }
}

#[cfg(test)]
mod test {
    use crate::props::{resolve_props, InertiaProp};
    use crate::req_type::{InertiaRequestType, PartialComponent};
    use crate::{Component, InertiaPage};
    use actix_web::test;
    use serde::Serialize;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;

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

        let mut props = HashMap::<String, InertiaProp>::new();
        props.insert(
            "auth".into(),
            InertiaProp::Data(json!({"name": "John Doe"})),
        );
        props.insert(
            "categories".into(),
            InertiaProp::Data(vec!["foo".to_string(), "bar".to_string()].into()),
        );
        props.insert(
            "events".into(),
            InertiaProp::Data(
                serde_json::to_value(vec![serde_json::to_value(event).unwrap()]).unwrap(),
            ),
        );

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
        let mut props = HashMap::<String, InertiaProp>::new();
        props.insert(
            "radioStatus".into(),
            InertiaProp::Demand(Arc::new(|| json!({"announcer": "John Doe"}))),
        );
        props.insert(
            "categories".into(),
            InertiaProp::Data(vec!["foo".to_string(), "bar".to_string()].into()),
        );

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
}
