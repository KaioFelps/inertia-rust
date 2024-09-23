use serde::Serialize;
use serde_json::{Map, Value};
use crate::error::InertiaError;

pub(crate) fn convert_struct_to_map<T>(s: T) -> Result<Map<String, Value>, InertiaError>
where T: Serialize
{
    let struct_as_value = match serde_json::to_value(s) {
        Ok(value) => value,
        Err(_) => {
            return Err(InertiaError::SerializationError("Struct is not JSON serializable.".into()))
        },
    };

    let value_as_map = match serde_json::from_value(struct_as_value) {
        Ok(value) => value,
        Err(err) => {
            return Err(InertiaError::SerializationError(format!("Failed to serialize struct as map: {}", err.to_string())));
        }
    };

    return Ok(value_as_map);
}

pub(crate) fn convert_struct_to_stringified_json<T>(s: T) -> Result<String, InertiaError>
where T: Serialize
{
    let map = convert_struct_to_map(s)?;
    return match serde_json::to_string(&map) {
        Ok(json) => Ok(json),
        Err(err) => Err(InertiaError::SerializationError(format!("Failed to serialize map to json: {}", err.to_string()))),
    };
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
