# Testing

To make testing even easier, Inertia provide a way of getting the inertia page as an object, so that you
can make assertions over it and ensure the expected response is being sent.

When making the request, call `.inertia()` from the `TestRequest` builder. This ensures that the response
will be a JSON rather than an HTML.

Then, simply call `.into_assertable_inertia()` from the response.

```rust
use inertia_rust::hashmap;
use inertia_rust::test::{InertiaTestRequest, IntoAssertableInertia};

#[tokio::test]
async fn test() {
    let app = actix_web::test::init_service(/* ... */).await;

    let request = actix_web::test::TestRequest::get()
        .inertia()
        .uri("/")
        .to_request();

    let inertia_page = actix_web::test::call_service(&app, request)
        .await
        .into_assertable_inertia();

    assert_eq!("/", inertia_page.get_url());
    assert_eq!("Index", inertia_page.get_component());
    assert_eq!(
        &Map::from(hashmap!["foo" => "bar".to_string().into()]),
        assertable.get_props()
    );
}
```
