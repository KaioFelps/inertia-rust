mod common;

use std::collections::HashMap;
use actix_web::{dev::{ServiceFactory, ServiceRequest, ServiceResponse}, get, web::Data, App, HttpRequest, HttpResponse};
use common::template_resolver::{mocked_resolver, EXPECTED_RENDER, EXPECTED_RENDER_W_PROPS};
use inertia_rust::{Component, Inertia, InertiaProp, InertiaProps, InertiaVersion};

fn super_trim(text: String) -> String {
    text.trim()
        .replace("\r\n", "")
        .replace("\n", "")
        .replace("\t", "")
}

// region: --- Service

#[get("/")]
async fn home(req: HttpRequest) -> HttpResponse {
    let response = inertia_rust::render::<()>(&req, Component("Index".into())).await;
    if response.is_ok() {
        return response.unwrap();
    }

    let err = response.unwrap_err();
    println!("{:#?}", err);
    return HttpResponse::InternalServerError().finish();
}

#[get("/withprops")]
async fn with_props(req: HttpRequest) -> HttpResponse {
    let mut props: InertiaProps = HashMap::new();
    props.insert("user".to_string(), InertiaProp::Always("John Doe".into()));

    let response = inertia_rust::render_with_props::<()>(
        &req,
        Component("Index".into()),
        props,
    ).await;
    if response.is_ok() { return response.unwrap(); }

    let err = response.unwrap_err();
    println!("{:#?}", err);
    return HttpResponse::InternalServerError().finish();
}

async fn generate_actix_app() -> App<impl ServiceFactory<
    ServiceRequest,
    Config = (),
    Response = ServiceResponse,
    Error = actix_web::Error,
    InitError = ()
>> {
    let inertia = Inertia::new(
        "https://inertiajs.com",
        InertiaVersion::Literal("v1.0.0".into()),
        // "tests/common/root_layout.html",
        "tests/common/root_layout.html",
        &mocked_resolver,
        &()
    );

    App::new()
        .app_data(Data::new(inertia))
        .service(home)
        .service(with_props)
}

// endregion: --- Service

// region: --- Tests


#[tokio::test]
async fn test_render() {
    let app =
        actix_web::test::init_service(generate_actix_app().await).await;

    let req = actix_web::test::TestRequest::get().uri("/").to_request();
    let resp = actix_web::test::call_service(&app, req).await;
    
    assert_eq!(200u16, resp.status().as_u16());

    let body = resp.into_body();
    let body_bytes = actix_web::body::to_bytes(body).await.unwrap();
    let html_body = String::from_utf8(body_bytes.to_vec()).unwrap();

    assert_eq!(super_trim(EXPECTED_RENDER.into()), super_trim(html_body));
}

#[tokio::test]
async fn test_render_with_props() {
    let app =
        actix_web::test::init_service(generate_actix_app().await).await;

    let req = actix_web::test::TestRequest::get().uri("/withprops").to_request();
    let resp = actix_web::test::call_service(&app, req).await;

    assert_eq!(200u16, resp.status().as_u16());

    let body = resp.into_body();
    let body_bytes = actix_web::body::to_bytes(body).await.unwrap();
    let html_body = String::from_utf8(body_bytes.to_vec()).unwrap();

    assert_eq!(super_trim(EXPECTED_RENDER_W_PROPS.into()), super_trim(html_body));
}

// endregion: --- Tests
