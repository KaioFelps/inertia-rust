use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub type DeserializedDeferredProps = Option<HashMap<String, Vec<String>>>;

#[derive(Deserialize, PartialEq, Clone, Debug)]
#[allow(unused)]
pub struct AssertableInertia {
    pub component: String,
    pub props: Map<String, Value>,
    pub url: String,
    pub version: Option<String>,

    #[serde(rename = "clearHistory")]
    pub clear_history: bool,

    #[serde(rename = "encryptHistory")]
    pub encrypt_history: bool,

    #[serde(rename = "deferredProps")]
    pub deferred_props: DeserializedDeferredProps,

    #[serde(rename = "mergeProps")]
    pub merge_props: Option<Vec<String>>,
}

pub trait InertiaTestRequest {
    /// Injects the Inertia header to the request, making the incoming response
    /// being a JSON instead of an HTML body.
    ///
    /// This allows the response to be serialized to an `AssertableInertia` instance.
    fn inertia(self) -> Self;
}

pub trait IntoAssertableInertia {
    /// Extract an `AssertableInertia` instance from `self`. This is only intended for
    /// testing and may interrupt your application at any moment, since it may panic on
    /// any error.
    ///
    /// # Panic
    /// If any error comes from an `Result` it will panic, since it will always dangerously
    /// unwrap the result.
    fn into_assertable_inertia(self) -> AssertableInertia;
}

impl AssertableInertia {
    pub fn get_component(&self) -> &str {
        &self.component
    }

    pub fn get_props(&self) -> &Map<String, Value> {
        &self.props
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_version(&self) -> Option<&String> {
        self.version.as_ref()
    }

    pub fn get_clear_history(&self) -> bool {
        self.clear_history
    }

    pub fn get_encrypt_history(&self) -> bool {
        self.encrypt_history
    }

    pub fn get_deferred_props(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.deferred_props.as_ref()
    }

    pub fn get_merge_props(&self) -> Option<&Vec<String>> {
        self.merge_props.as_ref()
    }
}
