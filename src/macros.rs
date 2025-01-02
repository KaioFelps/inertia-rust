#[macro_export]
macro_rules! hashmap {
    () => ( std::collections::HashMap::new() );
    ($( $key: expr => $value: expr ),+ $(,)?) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key, $value);
            )*
            map
        }
    };
}

#[cfg(test)]
mod test {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use crate::InertiaProp;

    #[test]
    fn test_hashmap_macro() {
        let mut manual_hashmap = HashMap::new();
        manual_hashmap.insert("foo".to_string(), 10);
        manual_hashmap.insert("bar".to_string(), 25);
        manual_hashmap.insert("baz".to_string(), 49020);

        let macro_hashmap = hashmap![
            "foo".to_string() => 10,
            "bar".to_string() => 25,
            "baz".to_string() => 49020,
        ];

        assert_eq!(manual_hashmap, macro_hashmap);
        assert_eq!(
            HashMap::<_, _>::new() as HashMap<String, String>,
            hashmap![]
        );
    }
}
