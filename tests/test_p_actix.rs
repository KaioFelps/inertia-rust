mod common;

use actix_web::dev::{forward_ready, Service, Transform};
use actix_web::http::header::TryIntoHeaderPair;
use actix_web::{
    body::MessageBody,
    delete,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    get,
    http::StatusCode,
    post, put,
    web::{Data, Query, Redirect},
    App, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use common::template_resolver::{get_dynamic_csr_expect, MockedTemplateResolver};
use futures::future::{ready, LocalBoxFuture, Ready};
use inertia_rust::{
    actix::{EncryptHistoryMiddleware, InertiaHeader, InertiaMiddleware},
    hashmap, prop_resolver, InertiaConfigBuilder, InertiaFacade, InertiaPage, InertiaService,
    InertiaSessionToReflash, InertiaTemporarySession,
};
use inertia_rust::{Component, Inertia, InertiaConfig, InertiaProp, InertiaVersion};
use serde::Deserialize;
use serde_json::{json, to_value, Map};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

const TEST_INERTIA_VERSION: &str = "v1.0.0";
static SESSIONS_STORAGE: OnceLock<Arc<Mutex<Vec<InertiaTemporarySession>>>> = OnceLock::new();
static TIMES_DEFERRED_RESOLVER_HAS_EXECUTED: OnceLock<Arc<Mutex<u32>>> = OnceLock::new();

// region: --- Helpers

fn super_trim(text: String) -> String {
    text.trim()
        .replace("\r\n", "")
        .replace("\n", "")
        .replace("\t", "")
}

fn request_as_bytes_vec(response: ServiceResponse) -> Vec<u8> {
    response.into_body().try_into_bytes().unwrap().to_vec()
}

fn get_inertia_config() -> InertiaConfigBuilder<&'static str> {
    InertiaConfig::builder()
        .set_url("https://inertiajs.com")
        .set_version(InertiaVersion::Literal(TEST_INERTIA_VERSION))
        .set_template_path("tests/common/root_layout.html")
        .set_template_resolver(Box::new(MockedTemplateResolver))
}

// endregion: --- Helpers

// region: --- Service

// region: --- Routes
#[get("/")]
async fn home(req: HttpRequest) -> impl Responder {
    let response = Inertia::render(&req, Component("Index".into())).await;
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
    Inertia::render_with_props(
        &req,
        "Index".into(),
        HashMap::from([("user", InertiaProp::always("John Doe"))]),
    )
    .await
}

#[derive(Deserialize)]
struct MergeAndDeferredPropsQuery {
    per_page: Option<usize>,
    page: Option<usize>,
}

#[get("/merge_and_deferred_props")]
async fn merge_and_deferred_props(
    req: HttpRequest,
    query: Query<MergeAndDeferredPropsQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(3);

    let users = Arc::new(["user1", "user2", "user3", "user4", "user5"]);
    let permissions = ["read", "update", "delete", "create"];

    Inertia::render_with_props(
        &req,
        "Index".into(),
        hashmap![
            "authUser" => InertiaProp::data(""),
            // let's pretend this is a very heavy operation!
            // so it make sense to defer it
            "users" => InertiaProp::defer(prop_resolver!(
                    let users_clone = users.clone(); {
                    let counter = TIMES_DEFERRED_RESOLVER_HAS_EXECUTED.get_or_init(|| Arc::new(Mutex::new(0)));
                    *counter.lock().unwrap() += 1;

                    to_value(users_clone
                    .clone()
                    .iter()
                    .skip((page -1)* per_page)
                    .take(per_page)
                    .cloned()
                    .collect::<Vec<_>>())
                    .unwrap()
                }))
                .into_mergeable(),
                "permissions" => InertiaProp::merge(permissions.into_iter().skip((page-1)*per_page).take(per_page).collect::<Vec<_>>())
        ],
    )
    .await
}

#[get("/location")]
async fn location(req: HttpRequest) -> impl Responder {
    Inertia::location(&req, "https://inertiajs.com")
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

#[post("/redirect/back/with/session")]
async fn redirect_back_with_reflash(req: HttpRequest) -> impl Responder {
    let mut errors = Map::new();

    errors.insert(
        "name".into(),
        to_value("Name too generic: 'John Doe' >:(").unwrap(),
    );

    let session = InertiaTemporarySession {
        errors: Some(errors),
        prev_req_url: "http://localhost:3000/foo".into(),
    };

    // "reflashes"
    req.extensions_mut()
        .insert(InertiaSessionToReflash(session));

    Inertia::back(&req)
}

#[post("/redirect/back/with/header")]
async fn redirect_back_with_header(req: HttpRequest) -> impl Responder {
    Inertia::back(&req)
}

#[get("/encrypt/method")]
async fn encrypt_with_method(req: HttpRequest) -> impl Responder {
    Inertia::encrypt_history(&req, true);
    Inertia::render(&req, "Foo".into()).await
}

#[get("/encrypt/overwrites")]
async fn encrypt_ovewrites(req: HttpRequest) -> impl Responder {
    Inertia::encrypt_history(&req, false);
    Inertia::render(&req, "Foo".into()).await
}

#[get("/encrypt/clearhistory")]
async fn encrypt_clear_history(req: HttpRequest) -> impl Responder {
    Inertia::clear_history(&req);
    Inertia::render(&req, "Foo".into()).await
}

// endregion: --- Routes

// region: --- Middleware
struct ReflashTemporarySessionMiddleware;

impl<S, B> Transform<S, ServiceRequest> for ReflashTemporarySessionMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = ReflashTemporarySessionService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ReflashTemporarySessionService { service }))
    }
}

struct ReflashTemporarySessionService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ReflashTemporarySessionService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let before_prev_req_url = "http://localhost:3000/bar";
        let prev_url = "http://localhost:3000/foo";

        let session = SESSIONS_STORAGE.get().unwrap().lock().unwrap();
        let session = session.first();
        if let Some(session) = session {
            req.extensions_mut().insert(session.clone());
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            let session = SESSIONS_STORAGE.get().unwrap();
            let req = res.request();
            let inertia_session = req.extensions_mut().remove::<InertiaSessionToReflash>();

            let (prev_url, _, errors) =
                if let Some(InertiaSessionToReflash(inertia_session)) = inertia_session {
                    (
                        before_prev_req_url,
                        inertia_session.prev_req_url,
                        inertia_session.errors,
                    )
                } else {
                    (prev_url, req.uri().to_string(), None)
                };

            session.lock().unwrap().push(InertiaTemporarySession {
                errors,
                prev_req_url: prev_url.into(),
            });

            Ok(res)
        })
    }
}

// endregion: --- Middleware

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

    let inertia = Inertia::new(get_inertia_config().build()).unwrap();

    App::new()
        .app_data(Data::new(inertia))
        .service(home)
        .service(with_props)
        .service(put_redirect)
        .service(post_redirect)
        .service(delete_redirect)
        .inertia_route("/withservice", "Index")
        .service(merge_and_deferred_props)
        .service(location)
        .service(encrypt_with_method)
        .service(encrypt_ovewrites)
        .service(encrypt_clear_history)
        .service(redirect_back_with_header)
        .service(redirect_back_with_reflash)
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
async fn test_location() {
    let app = actix_web::test::init_service(generate_actix_app().await).await;

    let request = actix_web::test::TestRequest::get()
        .uri("/location")
        .insert_header(InertiaHeader::Inertia.convert())
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .to_request();

    let response = actix_web::test::call_service(&app, request).await;

    assert_eq!(409u16, response.status().as_u16());
    assert_eq!(
        "https://inertiajs.com",
        response.headers().get("x-inertia-location").unwrap()
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
    const TEST_SHARED_PROPERTY_KEY: &str = "sharedProperty";
    const TEST_SHARED_PROPERTY_VALUE: &str = "Some amazing value!";

    let app = actix_web::test::init_service(generate_actix_app().await.wrap(
        InertiaMiddleware::new().with_shared_props(Arc::new(|_req| {
            hashmap![
            TEST_SHARED_PROPERTY_KEY => InertiaProp::Always(TEST_SHARED_PROPERTY_VALUE.into()),
            ]
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
        TEST_SHARED_PROPERTY_VALUE,
        json_body.get_props().get(TEST_SHARED_PROPERTY_KEY).unwrap()
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
    let app = actix_web::test::init_service(
        generate_actix_app()
            .await
            .wrap(ReflashTemporarySessionMiddleware)
            .wrap(InertiaMiddleware::new()),
    )
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
    // assert_eq!(&errors, storage.lock().unwrap()[0].errors.as_ref().unwrap());
}

#[tokio::test]
async fn test_defer_and_merge_props() {
    const ROUTE: &str = "/merge_and_deferred_props";
    let app = actix_web::test::init_service(generate_actix_app().await).await;

    let initial_standard_request = actix_web::test::TestRequest::get()
        .uri(ROUTE)
        .insert_header(InertiaHeader::Inertia.convert())
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .to_request();

    // in order to retrieve the deferred "users" prop
    let initial_partial_request = actix_web::test::TestRequest::get()
        .uri(&format!("{}?page=2", ROUTE))
        .insert_header(InertiaHeader::Inertia.convert())
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .insert_header(InertiaHeader::InertiaPartialData(vec!["users"]).convert())
        .insert_header(InertiaHeader::InertiaPartialComponent("Index".into()).convert())
        .to_request();

    let standard_body = actix_web::test::call_service(&app, initial_standard_request)
        .await
        .into_body()
        .try_into_bytes()
        .unwrap()
        .to_vec();

    let partial_body = actix_web::test::call_service(&app, initial_partial_request)
        .await
        .into_body()
        .try_into_bytes()
        .unwrap()
        .to_vec();

    let standard_body: InertiaPage = serde_json::from_slice(&standard_body[..]).unwrap();
    let partial_body: InertiaPage = serde_json::from_slice(&partial_body[..]).unwrap();

    assert!(["users", "permissions"].iter().all(|prop| standard_body
        .get_merge_props()
        .as_ref()
        .is_some_and(|props| props.contains(prop))));

    assert!(standard_body
        .get_deferred_props()
        .as_ref()
        .is_some_and(|props| props
            .get("default")
            .is_some_and(|default_props| default_props.eq(&["users"]))));

    assert!(standard_body
        .get_props()
        .get("permissions")
        .is_some_and(
            |permissions| ["read", "delete", "update"]
                .iter()
                .all(|permission| permissions
                    .as_array()
                    .unwrap()
                    .contains(&to_value(permission).unwrap()))
        ));

    assert!(partial_body
        .get_props()
        .get("users")
        .is_some_and(|users_prop| ["user4", "user5"].iter().all(|user| users_prop
            .as_array()
            .unwrap()
            .contains(&to_value(user).unwrap()))));

    assert_eq!(
        1,
        *TIMES_DEFERRED_RESOLVER_HAS_EXECUTED
        .get_or_init(|| Arc::new(Mutex::new(0)))
        .lock()
        .unwrap(),
        "Deferred Resolver should have been called only once, since only one request has required it's group."
    );
}

// endregion: --- Tests

// region: --- History Encryption Tests

#[tokio::test]
async fn test_history_encrypt_method() {
    let app = actix_web::test::init_service(generate_actix_app().await).await;

    let req = actix_web::test::TestRequest::get()
        .uri("/encrypt/method")
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .insert_header(InertiaHeader::Inertia.convert())
        .to_request();

    let body = actix_web::test::call_service(&app, req)
        .await
        .into_body()
        .try_into_bytes()
        .unwrap()
        .to_vec();

    let body: InertiaPage = serde_json::from_slice(&body[..]).unwrap();

    assert!(body.get_encrypt_history());
}

#[tokio::test]
async fn test_history_encrypt_middleware() {
    let app = actix_web::test::init_service(
        generate_actix_app()
            .await
            .wrap(EncryptHistoryMiddleware::new()),
    )
    .await;

    let req = actix_web::test::TestRequest::get()
        .uri("/")
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .insert_header(InertiaHeader::Inertia.convert())
        .to_request();

    let body = request_as_bytes_vec(actix_web::test::call_service(&app, req).await);
    let body: InertiaPage = serde_json::from_slice(&body[..]).unwrap();

    assert!(body.get_encrypt_history());
}

#[tokio::test]
async fn test_history_encryt_method_overwrites_middleware() {
    let app = actix_web::test::init_service(
        generate_actix_app()
            .await
            .wrap(EncryptHistoryMiddleware::new()),
    )
    .await;

    let req = actix_web::test::TestRequest::get()
        .uri("/encrypt/overwrites")
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .insert_header(InertiaHeader::Inertia.convert())
        .to_request();

    let body = request_as_bytes_vec(actix_web::test::call_service(&app, req).await);
    let body: InertiaPage = serde_json::from_slice(&body[..]).unwrap();

    assert!(!body.get_encrypt_history());
}

#[tokio::test]
async fn test_history_encrypt_from_config() {
    let inertia = actix_web::web::Data::new(
        Inertia::new(get_inertia_config().encrypt_history().build()).unwrap(),
    );

    let app = App::new().app_data(inertia).inertia_route("/", "Index");
    let app = actix_web::test::init_service(app).await;

    let req = actix_web::test::TestRequest::get()
        .uri("/")
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .insert_header(InertiaHeader::Inertia.convert())
        .to_request();

    let body = request_as_bytes_vec(actix_web::test::call_service(&app, req).await);
    let body: InertiaPage = serde_json::from_slice(&body[..]).unwrap();

    assert!(body.get_encrypt_history());
}

#[tokio::test]
async fn test_history_encrypt_overwrites_config() {
    let inertia = actix_web::web::Data::new(
        Inertia::new(get_inertia_config().encrypt_history().build()).unwrap(),
    );

    let app = App::new().app_data(inertia).service(encrypt_ovewrites);
    let app = actix_web::test::init_service(app).await;

    let req = actix_web::test::TestRequest::get()
        .uri("/encrypt/overwrites")
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .insert_header(InertiaHeader::Inertia.convert())
        .to_request();

    let body = request_as_bytes_vec(actix_web::test::call_service(&app, req).await);
    let body: InertiaPage = serde_json::from_slice(&body[..]).unwrap();

    assert!(!body.get_encrypt_history());
}

#[tokio::test]
async fn test_history_encrypt_method_overwrites_everything() {
    let inertia = actix_web::web::Data::new(
        Inertia::new(get_inertia_config().encrypt_history().build()).unwrap(),
    );

    let app = App::new()
        .app_data(inertia)
        .wrap(EncryptHistoryMiddleware::new())
        .service(encrypt_ovewrites);

    let app = actix_web::test::init_service(app).await;

    let req = actix_web::test::TestRequest::get()
        .uri("/encrypt/overwrites")
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .insert_header(InertiaHeader::Inertia.convert())
        .to_request();

    let body = request_as_bytes_vec(actix_web::test::call_service(&app, req).await);
    let body: InertiaPage = serde_json::from_slice(&body[..]).unwrap();

    assert!(!body.get_encrypt_history());
}

#[tokio::test]
async fn test_clear_history() {
    let app = actix_web::test::init_service(generate_actix_app().await).await;
    let req = actix_web::test::TestRequest::get()
        .uri("/encrypt/clearhistory")
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .insert_header(InertiaHeader::Inertia.convert())
        .to_request();

    let body = request_as_bytes_vec(actix_web::test::call_service(&app, req).await);
    let body: InertiaPage = serde_json::from_slice(&body[..]).unwrap();

    assert!(body.get_clear_history());
}

// endregion: --- History Encryption Tests

// region: --- Redirect Back Tests

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn test_redirect_back_with_reflash_session() {
    let app = actix_web::test::init_service(
        generate_actix_app()
            .await
            .wrap(ReflashTemporarySessionMiddleware),
    )
    .await;

    let req = actix_web::test::TestRequest::post()
        .uri("/redirect/back/with/session")
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .insert_header(InertiaHeader::Inertia.convert())
        .to_request();

    let response = actix_web::test::call_service(&app, req).await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(
        response
            .headers()
            .get(actix_web::http::header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap(),
        "http://localhost:3000/bar"
    );
}

#[tokio::test]
async fn test_redirect_back_with_referer_header() {
    let app = actix_web::test::init_service(generate_actix_app().await).await;

    let req = actix_web::test::TestRequest::post()
        .uri("/redirect/back/with/header")
        .insert_header(InertiaHeader::Version(TEST_INERTIA_VERSION).convert())
        .insert_header(InertiaHeader::Inertia.convert())
        .insert_header(
            (
                actix_web::http::header::REFERER,
                "http://localhost:3000/bar",
            )
                .try_into_pair()
                .unwrap(),
        )
        .to_request();

    let response = actix_web::test::call_service(&app, req).await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(
        response
            .headers()
            .get(actix_web::http::header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap(),
        "http://localhost:3000/bar"
    );
}

// endregion: --- Redirect Back Tests
