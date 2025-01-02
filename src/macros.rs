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

#[macro_export]
macro_rules! prop_resolver {
    () => (std::sync::Arc::new(move || Box::pin(async move { () })));

    ($($clone_stmt: stmt),+; $block: block) => {{
        std::sync::Arc::new(move || {
            $($clone_stmt)+
            Box::pin(async move $block)
        })
    }};

    ($block: block) => {{
        std::sync::Arc::new(move || Box::pin(async move $block))
    }};
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

    async fn an_async_operation() {}

    #[tokio::test]
    async fn test_prop_resolver_with_moving_let() {
        let message = Arc::new("Super important and often used message!");
        let counter = Arc::new(Mutex::new(1));

        let counter_clone = counter.clone();
        let message_clone = message.clone();

        let prop_with_macro = InertiaProp::lazy(prop_resolver!(
            let counter_clone = counter_clone.clone(),
            let message_clone = message_clone.clone();
            {
                an_async_operation().await;
                format!("{} {}", message_clone, *counter_clone.lock().unwrap()).into()
            }
        ));

        let counter_clone = counter.clone();
        let message_clone = message.clone();

        let prop_with_arc = InertiaProp::lazy(Arc::new(move || {
            let counter_clone = counter_clone.clone();
            let message_clone = message_clone.clone();

            Box::pin(async move {
                an_async_operation().await;
                format!("{} {}", message_clone, *counter_clone.lock().unwrap()).into()
            })
        }));

        assert_eq!(
            prop_with_arc.resolve_unconditionally().await,
            prop_with_macro.resolve_unconditionally().await
        );
    }
}
