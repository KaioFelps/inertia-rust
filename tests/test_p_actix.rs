mod common;

use actix_web::{
    body::MessageBody,
    delete,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    get,
    http::StatusCode,
    post, put,
    web::{Data, Redirect},
    App, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use common::template_resolver::{get_dynamic_csr_expect, mocked_resolver};
use inertia_rust::{
    actix::{render, render_with_props, InertiaHeader, InertiaMiddleware},
    InertiaPage, InertiaService, InertiaTemporarySession,
};
use inertia_rust::{Component, Inertia, InertiaConfig, InertiaProp, InertiaProps, InertiaVersion};
use serde_json::{json, Map};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

const TEST_INERTIA_VERSION: &str = "v1.0.0";
static SESSIONS_STORAGE: OnceLock<Arc<Mutex<Vec<InertiaTemporarySession>>>> = OnceLock::new();

fn super_trim(text: String) -> String {
    text.trim()
        .replace("\r\n", "")
        .replace("\n", "")
        .replace("\t", "")
}

// region: --- Service

#[get("/")]
async fn home(req: HttpRequest) -> impl Responder {
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
async fn with_props(req: HttpRequest) -> impl Responder {
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

#[put("/redirect")]
async fn put_redirect() -> impl Responder {
    Redirect::to("/").using_status_code(StatusCode::MOVED_PERMANENTLY)
}

#[post("/redirect")]
async fn post_redirect() -> impl Responder {
    Redirect::to("/").see_other()
}

#[delete("redirect")]
async fn delete_redirect() -> impl Responder {
    Redirect::to("/").using_status_code(StatusCode::FOUND)
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
    let _ = SESSIONS_STORAGE.get_or_init(|| Arc::new(Mutex::new(Vec::new())));

    let inertia = Inertia::new(
        InertiaConfig::builder()
            .set_url("https://inertiajs.com")
            .set_version(InertiaVersion::Literal(TEST_INERTIA_VERSION))
            .set_template_path("tests/common/root_layout.html")
            .set_template_resolver(&mocked_resolver)
            .set_template_resolver_data(&())
            .set_reflash_fn(Box::new(move |session| {
                if let Some(session) = session {
                    SESSIONS_STORAGE
                        .get()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .push(session);
                }

                Ok(())
            }))
            .build(),
    )
    .unwrap();

    App::new()
        .app_data(Data::new(inertia))
        .service(home)
        .service(with_props)
        .service(put_redirect)
        .service(post_redirect)
        .service(delete_redirect)
        .inertia_route::<()>("/withservice", "Index")
}

// endregion: --- Service

// region: --- Tests

#[tokio::test]
async fn test_assets_version_redirect() {
    let app =
        actix_web::test::init_service(generate_actix_app().await.wrap(InertiaMiddleware::new()))
            .await;

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

    assert_eq!(
        get_dynamic_csr_expect("/", "{}", "Index", TEST_INERTIA_VERSION),
        super_trim(html_body)
    );
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
        get_dynamic_csr_expect(
            "/withprops",
            &json!({"user": "John Doe"}).to_string(),
            "Index",
            TEST_INERTIA_VERSION
        ),
        super_trim(html_body)
    );
}

#[tokio::test]
async fn test_shared_props() {
    let test_shared_property_key = "sharedProperty";
    let test_shared_property_value = "Some amazing value!";

    let app = actix_web::test::init_service(generate_actix_app().await.wrap(
        InertiaMiddleware::new().with_shared_props(Arc::new(|_req| {
            let mut shared_props = HashMap::new();
            shared_props.insert(
                test_shared_property_key.to_string(),
                InertiaProp::Always(test_shared_property_value.into()),
            );

            shared_props
        })),
    ))
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

#[tokio::test]
async fn test_inertia_route_service() {
    let app = actix_web::test::init_service(generate_actix_app().await).await;

    let req = actix_web::test::TestRequest::get()
        .uri("/withservice")
        .to_request();

    let resp = actix_web::test::call_service(&app, req).await;

    assert_eq!(200u16, resp.status().as_u16());

    let body = String::from_utf8(resp.into_body().try_into_bytes().unwrap().to_vec()).unwrap();

    assert_eq!(
        get_dynamic_csr_expect("/withservice", "{}", "Index", TEST_INERTIA_VERSION),
        super_trim(body)
    );
}

#[tokio::test]
async fn test_inertia_middleware() {
    let app =
        actix_web::test::init_service(generate_actix_app().await.wrap(InertiaMiddleware::new()))
            .await;

    let req_put = actix_web::test::TestRequest::put()
        .uri("/redirect")
        .to_request();

    let req_post = actix_web::test::TestRequest::post()
        .uri("/redirect")
        .to_request();

    let req_delete = actix_web::test::TestRequest::delete()
        .uri("/redirect")
        .to_request();

    let resp_put = actix_web::test::call_service(&app, req_put).await;
    let resp_post = actix_web::test::call_service(&app, req_post).await;
    let resp_delete = actix_web::test::call_service(&app, req_delete).await;

    assert_eq!(303u16, resp_put.status().as_u16());
    assert_eq!(303u16, resp_post.status().as_u16());
    assert_eq!(303u16, resp_delete.status().as_u16());
}

#[tokio::test]
async fn test_inertia_temporary_sessions() {
    let app =
        actix_web::test::init_service(generate_actix_app().await.wrap(InertiaMiddleware::new()))
            .await;

    let request = actix_web::test::TestRequest::get()
        .uri("/withprops")
        .insert_header(InertiaHeader::Version("wrong_version").convert())
        .insert_header(InertiaHeader::Inertia.convert())
        .to_request();

    let mut errors = Map::new();
    errors.insert("foo".into(), "We are enemies, we are fooes...".into());

    request.extensions_mut().insert(InertiaTemporarySession {
        errors: Some(errors.clone()),
        prev_req_url: "/".into(),
    });

    // as wrong version has been set, it will force a refresh.
    // the mocked reflash method should be called, putting the above temporary session inside the static
    // list
    let response = actix_web::test::call_service(&app, request).await;
    assert_eq!(409u16, response.status().as_u16());

    let storage = SESSIONS_STORAGE.get().unwrap();

    assert!(!storage.lock().unwrap().is_empty());
    assert_eq!(&errors, storage.lock().unwrap()[0].errors.as_ref().unwrap());
}

// endregion: --- Tests
