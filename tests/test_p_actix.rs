mod common;

use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    get,
    web::Data,
    App, HttpRequest, HttpResponse,
};
use common::template_resolver::{mocked_resolver, EXPECTED_RENDER, EXPECTED_RENDER_W_PROPS};
use inertia_rust::{
    actix::{render, render_with_props, InertiaHeader, InertiaMiddleware},
    InertiaPage,
};
use inertia_rust::{Component, Inertia, InertiaConfig, InertiaProp, InertiaProps, InertiaVersion};
use std::{collections::HashMap, sync::Arc};

const TEST_INERTIA_VERSION: &str = "v1.0.0";

fn super_trim(text: String) -> String {
    text.trim()
        .replace("\r\n", "")
        .replace("\n", "")
        .replace("\t", "")
}

// region: --- Service

#[get("/")]
async fn home(req: HttpRequest) -> HttpResponse {
    let response = render::<()>(&req, Component("Index".into())).await;
    match response {
        Ok(response) => response,
        Err(error) => {
            log::error!("{:#?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/withprops")]
async fn with_props(req: HttpRequest) -> HttpResponse {
    let mut props: InertiaProps = HashMap::new();
    props.insert("user".to_string(), InertiaProp::Always("John Doe".into()));

    let response = render_with_props::<()>(&req, Component("Index".into()), props).await;

    match response {
        Ok(response) => response,
        Err(error) => {
            log::error!("{:#?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn generate_actix_app() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let inertia = Inertia::new(
        InertiaConfig::builder()
            .set_url("https://inertiajs.com")
            .set_version(InertiaVersion::Literal(TEST_INERTIA_VERSION))
            .set_template_path("tests/common/root_layout.html")
            .set_template_resolver(&mocked_resolver)
            .set_template_resolver_data(&())
            .build(),
    )
    .unwrap();

    App::new()
        .app_data(Data::new(inertia))
        .service(home)
        .service(with_props)
}

// endregion: --- Service

// region: --- Tests

#[tokio::test]
async fn test_assets_version_redirect() {
    let app = actix_web::test::init_service(generate_actix_app().await).await;

    let first_access_request = actix_web::test::TestRequest::get()
        .uri("/")
        .insert_header(InertiaHeader::Inertia.convert())
        .to_request();

    let n_access_request = actix_web::test::TestRequest::get()
        .uri("/")
        .insert_header(InertiaHeader::Inertia.convert())
        .insert_header(InertiaHeader::Version("any_other_version").convert())
        .to_request();

    let first_access_response = actix_web::test::call_service(&app, first_access_request).await;
    let n_access_response = actix_web::test::call_service(&app, n_access_request).await;

    assert_eq!(200u16, first_access_response.status().as_u16());
    assert!(first_access_response
        .headers()
        .get("x-inertia-location")
        .is_none());

    assert_eq!(409u16, n_access_response.status().as_u16());
    assert_eq!(
        "/",
        n_access_response
            .headers()
            .get("x-inertia-location")
            .unwrap()
    );
}

#[tokio::test]
async fn test_render() {
    let app = actix_web::test::init_service(generate_actix_app().await).await;

    let req = actix_web::test::TestRequest::get()
        .uri("/")
        .insert_header(InertiaHeader::Version("v1.0.0").convert())
        .to_request();
    let resp = actix_web::test::call_service(&app, req).await;

    assert_eq!(200u16, resp.status().as_u16());

    let body = resp.into_body();
    let body_bytes = actix_web::body::to_bytes(body).await.unwrap();
    let html_body = String::from_utf8(body_bytes.to_vec()).unwrap();

    assert_eq!(super_trim(EXPECTED_RENDER.into()), super_trim(html_body));
}

#[tokio::test]
async fn test_render_with_props() {
    let app = actix_web::test::init_service(generate_actix_app().await).await;

    let req = actix_web::test::TestRequest::get()
        .uri("/withprops")
        .append_header(InertiaHeader::Version("v1.0.0").convert())
        .to_request();
    let resp = actix_web::test::call_service(&app, req).await;

    assert_eq!(200u16, resp.status().as_u16());

    let body = resp.into_body();
    let body_bytes = actix_web::body::to_bytes(body).await.unwrap();
    let html_body = String::from_utf8(body_bytes.to_vec()).unwrap();

    assert_eq!(
        super_trim(EXPECTED_RENDER_W_PROPS.into()),
        super_trim(html_body)
    );
}

#[tokio::test]
async fn test_shared_props() {
    let test_shared_property_key = "sharedProperty";
    let test_shared_property_value = "Some amazing value!";

    let mut shared_props = HashMap::new();
    shared_props.insert(
        test_shared_property_key.to_string(),
        InertiaProp::Always(test_shared_property_value.into()),
    );

    let shared_props = Arc::new(shared_props);

    let app = actix_web::test::init_service(
        generate_actix_app()
            .await
            .wrap(InertiaMiddleware::new().with_shared_props(shared_props)),
    )
    .await;

    let req = actix_web::test::TestRequest::get()
        .uri("/")
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .insert_header(InertiaHeader::Inertia.convert())
        .to_request();

    let body = actix_web::test::call_service(&app, req)
        .await
        .into_body()
        .try_into_bytes()
        .unwrap()
        .to_vec();

    let json_body: InertiaPage = serde_json::from_slice(&body[..]).unwrap();

    assert_eq!(
        test_shared_property_value,
        json_body.get_props().get(test_shared_property_key).unwrap()
    );
}

// endregion: --- Tests
