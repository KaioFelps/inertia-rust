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

/// Response containing a valid Inertia Payload that will be used
/// by the Inertia client to render the components.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct InertiaPage {
    /// The name of the JavaScript page component.
    pub(crate) component: Component,
    /// The page props (data). A merge of page props and shared props.
    pub(crate) props: Map<String, Value>,
    /// Page's URL. Must be a valid href.
    // this is not the same as Inertia::url, that represents the application url.
    // this url represents the current request's url, i.e. the page url.
    pub(crate) url: String,
    /// Current assets version.
    pub(crate) version: Option<String>,
}

impl InertiaPage {
    /// Instantiates an Inertia Page object to sent as http response,
    /// according to [Inertia Protocol].
    ///
    /// [Inertia Protocol]: https://inertiajs.com/the-protocol
    ///
    /// # Arguments
    /// * `component`   -   The name of the javascript page component (e.g. "/Me").
    /// * `url`         -   The Inertia instance's url (the application URL). It can be a
    ///                     whole href or an absolute hostless path ("/me").
    /// * `version`     -   Current assets version. Used to assert assets are up-to-date. See
    ///                     [Inertia's assets versioning] page for more details.
    /// * `props`       -   A map of the page's props.
    ///
    /// [Inertia's assets versioning]: https://inertiajs.com/the-protocol#asset-versioning
    ///
    pub fn new(
        component: Component,
        url: String,
        // this indicates that the version str must live at least until the request is freed
        // from memory. This will happen because the version given is the Inertia::version, a
        // static living str.
        version: Option<String>,
        props: Map<String, Value>,
    ) -> Self {
        InertiaPage {
            component,
            url,
            props,
            version,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::props::InertiaProp;
    use crate::req_type::{InertiaRequestType, PartialComponent};
    use crate::{Component, InertiaPage};
    use actix_web::test;
    use serde::Serialize;
    use serde_json::json;
    use std::collections::HashMap;

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
            "/events/80".to_string(),
            Some("generated_version".into()),
            InertiaProp::resolve_props(props, req_type),
        );

        let json_page_example = json!({
          "component": "Events",
          "props": {
            // "auth": { "name": "John Doe" },              // NOT included
            // "categories": ["foo", "bar"],                // NOT included
            "events": [{"id": 1, "title": "Baile"}]      // included
          },
          "url": "/events/80",
          "version": "generated_version"
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
            InertiaProp::Demand(|| json!({"announcer": "John Doe"})),
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
            "/categories".to_string(),
            Some("generated_version".into()),
            InertiaProp::resolve_props(props, req_type),
        );

        let json_page_example = json!({
          "component": "Categories",
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
