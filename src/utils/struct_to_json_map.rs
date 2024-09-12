use serde::Serialize;
use serde_json::{Map, Value};
use crate::core::inertia_errors::InertiaErrors;

/// Converts a struct of type `T` into a serde_json::Map.
///
/// ## Errors
/// Results in `InertiaErrors` if the struct has any field that does not also implement
/// `Serialize`.
pub(crate) fn convert_struct_to_map<T>(s: T) -> Result<Map<String, Value>, InertiaErrors>
where T: Serialize
{
    let struct_as_value = match serde_json::to_value(s) {
        Ok(value) => value,
        Err(_) => {
            return Err(InertiaErrors::SerializationError("Struct is not JSON serializable.".into()))
        },
    };

    let value_as_map = match serde_json::from_value(struct_as_value) {
        Ok(value) => value,
        Err(err) => {
            return Err(InertiaErrors::SerializationError(format!("Failed to serialize struct as map: {}", err.to_string())));
        }
    };

    return Ok(value_as_map);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_convert_struct_to_map() {
        #[derive(serde::Serialize)]
        struct Foo {
            bar: u32,
            baz: bool,
        }

        #[derive(serde::Serialize)]
        struct Props {
            statement: String,
            foo: Foo,
        }

        let props = Props {
            statement: "Inertia slays!".into(),
            foo: Foo {
                bar: 2024,
                baz: true,
            }
        };

        let parsed_to_json_map = convert_struct_to_map(props).unwrap();

        assert_eq!(serde_json::to_string(&parsed_to_json_map).unwrap(), "{\"foo\":{\"bar\":2024,\"baz\":true},\"statement\":\"Inertia slays!\"}")
    }
}
