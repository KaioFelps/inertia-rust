use super::middleware::SharedProps;
use super::{headers, CustomViewData};
use crate::facade::InertiaFacade;
use crate::inertia::{
    Inertia, InertiaHttpRequest, InertiaResponder, InertiaService, ViewData, X_INERTIA,
};
use crate::props::InertiaProps;
use crate::props::{get_deferred_props, get_mergeable_props, resolve_props};
use crate::req_type::{InertiaRequestType, PartialComponent};
use crate::temporary_session::InertiaSessionToReflash;
use crate::utils::request_page_render;
use crate::{Component, InertiaError, InertiaPage, InertiaSSRPage, InertiaTemporarySession};
use actix_web::body::BoxBody;
use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::http::header::{self, HeaderName, HeaderValue, TryIntoHeaderValue};
use actix_web::http::StatusCode;
use actix_web::web::{Redirect, ServiceConfig};
use actix_web::{
    web, App, FromRequest, HttpMessage, HttpRequest, HttpResponse, HttpResponseBuilder, Responder,
    ResponseError,
};
use async_trait::async_trait;
use serde_json::{json, to_value, Map, Value};
use std::collections::HashMap;

impl Responder for InertiaPage<'_> {
    type Body = BoxBody;

    #[inline]
    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponseBuilder::new(StatusCode::OK)
            .body(BoxBody::new(serde_json::to_string(&self).unwrap()))
    }
}

#[async_trait(?Send)]
impl InertiaResponder<HttpResponse, HttpRequest, Redirect> for Inertia {
    #[inline]
    async fn inner_render<'b>(
        &'b self,
        req: &'b HttpRequest,
        component: Component,
        props: Option<InertiaProps<'b>>,
    ) -> Result<HttpResponse, InertiaError> {
        if let Some(forced_refresh) = self.check_and_handle_version_mismatch(req) {
            return Ok(forced_refresh);
        };

        let props = self.resolve_request_props(props, req);

        let url = req.uri().to_string();
        let req_type: InertiaRequestType = req.get_request_type()?;

        let page = InertiaPage {
            component,
            url: &url,
            version: Some(self.version),
            props: resolve_props(&props, &req_type).await?,
            merge_props: get_mergeable_props(&props, req.get_merge_props_to_be_reset()),
            deferred_props: get_deferred_props(&props, &req_type),
            clear_history: req.should_clear_history(),
            encrypt_history: req.should_encrypt_history(self.encrypt_history),
        };

        log::debug!("Starting render of page {page:#?}");

        let response = if req.is_inertia_request() {
            let mut response = page.respond_to(req);

            response.headers_mut().insert(
                header::CONTENT_TYPE,
                header::ContentType::json().try_into_value().unwrap(),
            );
            response
        } else {
            let view_data = self.resolve_view_data(req, page).await;
            let rendered_page = self.render_page(view_data).await?;

            HttpResponseBuilder::new(StatusCode::OK)
                .insert_header(header::ContentType::html())
                .body(rendered_page)
                .respond_to(req)
        };

        Ok(self.send_response(response))
    }

    #[inline]
    fn inner_location(req: &HttpRequest, url: &str) -> HttpResponse {
        if !req.is_inertia_request() {
            return HttpResponse::Found()
                .append_header((actix_web::http::header::LOCATION, url))
                .finish();
        }

        HttpResponseBuilder::new(StatusCode::CONFLICT)
            .append_header(headers::InertiaHeader::InertiaLocation(url).convert())
            .finish()
    }

    #[inline]
    fn inner_encrypt_history(req: &HttpRequest, encrypt: bool) {
        req.extensions_mut().insert(ShallEncryptHistory(encrypt));
    }

    #[inline]
    fn inner_clear_history(req: &HttpRequest) {
        req.extensions_mut().insert(ShallClearHistory);
    }

    #[inline]
    fn inner_back(&self, req: &HttpRequest) -> Redirect {
        let uri = if let Some(Ok(uri)) = req
            .headers()
            .get(HeaderName::from_static("referer"))
            .map(|header| header.to_str())
        {
            uri.to_owned()
        } else {
            req.extensions()
                .get::<InertiaTemporarySession>()
                .map(|session| session.prev_req_url.to_owned())
                .unwrap_or("/".to_owned())
        };

        Redirect::new(req.uri().to_string(), uri).using_status_code(StatusCode::FOUND)
    }

    #[inline]
    fn inner_back_with_errors<T: ToString>(
        &self,
        req: &HttpRequest,
        errors: HashMap<T, Value>,
    ) -> Redirect {
        if !errors.is_empty() {
            let mut errors_map = Map::new();

            for (key, value) in errors {
                match to_value(value) {
                    Ok(value) => {
                        errors_map.insert(key.to_string(), value);
                    }

                    Err(err) => {
                        log::error!("Failed to serialize session's error value: {}", err);
                        continue;
                    }
                }
            }

            req.extensions_mut()
                .insert(SessionErrors(resolve_session_errors(errors_map, req)));
        }

        self.inner_back(req)
    }
}

fn resolve_session_errors(errors: Map<String, Value>, req: &HttpRequest) -> Map<String, Value> {
    let error_bag_header = match req.headers().get(super::headers::X_INERTIA_ERROR_BAG) {
        Some(bag) => bag,
        None => return errors,
    };

    if let Ok(bag) = error_bag_header.to_str() {
        return Map::from_iter([(
            bag.to_string(),
            to_value(errors).unwrap_or_else(|err| {
                log::error!("Failed to serialize session errors: {}", err);
                json!({})
            }),
        )]);
    }

    log::warn!(
        "Received an invalid header {} value. Opting out of error bag.",
        super::headers::X_INERTIA_ERROR_BAG,
    );

    errors
}

impl ResponseError for InertiaError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR)
            .insert_header(actix_web::http::header::ContentType::json())
            .body(self.get_cause())
    }
}

impl InertiaService for &mut ServiceConfig {
    fn inertia_route(self, path: &str, component: &'static str) -> Self {
        self.route(
            path,
            web::get().to(move |req: HttpRequest| async move {
                Inertia::render(&req, component.into()).await
            }),
        );

        self
    }
}

impl<TApp> InertiaService for App<TApp>
where
    TApp: ServiceFactory<
        ServiceRequest,
        Config = (),
        Error = actix_web::error::Error,
        InitError = (),
    >,
{
    fn inertia_route(self, path: &str, component: &'static str) -> Self {
        self.route(
            path,
            web::get().to(move |req: HttpRequest| async move {
                Inertia::render(&req, component.into()).await
            }),
        )
    }
}

/// Contains the Errors from this session.
/// Might be added to the request extensions through `Inertia::back_with_errors` method.
/// Must be added to the sessions by your own custom middleware at the end of a request,
/// so that it is fetched on the subsequent request from that session.
pub struct SessionErrors(pub Map<String, Value>);
pub(crate) struct ShallClearHistory;
pub(crate) struct ShallEncryptHistory(pub bool);

impl InertiaHttpRequest for HttpRequest {
    fn is_inertia_request(&self) -> bool {
        match self.headers().get(headers::X_INERTIA) {
            None => false,
            Some(v) => !v.is_empty(),
        }
    }

    fn get_request_type(&self) -> Result<InertiaRequestType, InertiaError> {
        let header = match self.headers().get(headers::X_INERTIA_PARTIAL_COMPONENT) {
            Some(header) => header,
            None => return Ok(InertiaRequestType::Standard),
        };

        let component: Component = header
            .to_str()
            .map_err(|_| {
                InertiaError::SerializationError(format!(
                    "Failed to serialize header {}",
                    headers::X_INERTIA_PARTIAL_COMPONENT
                ))
            })?
            .into();

        let partial_component = PartialComponent {
            component,
            only: extract_partials_headers_content(self, &headers::X_INERTIA_PARTIAL_DATA)?,
            except: extract_partials_headers_content(self, &headers::X_INERTIA_PARTIAL_EXCEPT)?,
        };

        Ok(InertiaRequestType::Partial(partial_component))
    }

    fn check_inertia_version(&self, current_version: &str) -> bool {
        self.headers()
            .get(headers::X_INERTIA_VERSION)
            .is_none_or(|version| {
                version
                    .to_str()
                    .is_ok_and(|version| version == current_version)
            })
    }

    fn get_merge_props_to_be_reset(&self) -> Vec<&str> {
        self.headers()
            .get(headers::X_INERTIA_RESET)
            .map_or(vec![], |header| {
                header
                    .to_str()
                    .unwrap_or("")
                    .split(", ")
                    .collect::<Vec<_>>()
            })
    }

    fn should_clear_history(&self) -> bool {
        self.extensions().get::<ShallClearHistory>().is_some()
    }

    fn should_encrypt_history(&self, default: bool) -> bool {
        match self.extensions().get::<ShallEncryptHistory>() {
            Some(ShallEncryptHistory(should_encrypt)) => *should_encrypt,
            None => default,
        }
    }
}

fn extract_partials_headers_content(
    req: &HttpRequest,
    header_name: &HeaderName,
) -> Result<Vec<String>, InertiaError> {
    let partials = match req.headers().get(header_name) {
        None => return Ok(Vec::new()),
        Some(value) => value,
    };

    partials
        .to_str()
        .map(|partials| partials.split(",").map(|v| v.to_string()).collect())
        .map_err(|_| {
            InertiaError::HeaderError(format!(
                "Header {}'s value must contain only printable ASCII characters.",
                header_name,
            ))
        })
}

impl Inertia {
    async fn get_ssr_page(&self, page: &InertiaPage<'_>) -> Option<InertiaSSRPage> {
        let ssr_server_url = match &self.ssr_url {
            None => return None,
            Some(url) => url,
        };

        request_page_render(ssr_server_url, page)
            .await
            .map(Some)
            .unwrap_or_else(|err| {
                log::error!(
                    "[Inertia Rust] Error on server-side rendering page {} with page props {page:#?}: {}",
                    page.component.0,
                    err
                );

                None
            })
    }

    fn resolve_custom_view_data(&self, req: &HttpRequest, is_ssr: bool) -> Map<String, Value> {
        let mut custom_view_data = req
            .extensions_mut()
            .remove::<CustomViewData>()
            .map(|data| data.0)
            .unwrap_or_default();

        custom_view_data.insert("isSsr".into(), is_ssr.into());
        custom_view_data.insert("is_ssr".into(), is_ssr.into());

        custom_view_data
    }

    async fn resolve_view_data<'a>(
        &'a self,
        req: &'a HttpRequest,
        page: InertiaPage<'a>,
    ) -> ViewData<'a> {
        let ssr_page = self.get_ssr_page(&page).await;
        let custom_props = self.resolve_custom_view_data(req, ssr_page.is_some());

        ViewData {
            ssr_page,
            custom_props,
            page,
        }
    }

    async fn render_page(&self, view_data: ViewData<'_>) -> Result<String, InertiaError> {
        self.template_resolver.resolve_template(view_data).await
    }

    fn send_response(&self, mut response: HttpResponse) -> HttpResponse {
        let headers = response.headers_mut();

        let (x_inertia, x_inertia_value) = headers::InertiaHeader::Inertia.convert();

        headers.insert(x_inertia, x_inertia_value);
        headers.insert(header::VARY, HeaderValue::from_static(X_INERTIA));

        response
    }

    fn resolve_request_props<'a>(
        &'a self,
        props: Option<InertiaProps<'a>>,
        req: &HttpRequest,
    ) -> InertiaProps<'a> {
        let mut props = props.unwrap_or_default();

        let shared_props = req
            .extensions_mut()
            .remove::<SharedProps>()
            .map(|shared_props| shared_props.0);

        if let Some(shared_props) = shared_props {
            props.extend(shared_props);
        }

        props
    }
}

trait InertiaActixHelpers {
    fn check_and_handle_version_mismatch(&self, req: &HttpRequest) -> Option<HttpResponse>;
}

impl InertiaActixHelpers for Inertia {
    fn check_and_handle_version_mismatch(&self, req: &HttpRequest) -> Option<HttpResponse> {
        if req.is_inertia_request() && !req.check_inertia_version(self.version) {
            reflash_inertia_session(req);
            return Some(Inertia::location(req, &req.uri().to_string()));
        }

        None
    }
}

fn reflash_inertia_session(req: &HttpRequest) {
    let inertia_temporary_session = req.extensions_mut().remove::<InertiaTemporarySession>();

    if let Some(session) = inertia_temporary_session {
        req.extensions_mut()
            .insert(InertiaSessionToReflash(session));
    }
}

impl FromRequest for InertiaTemporarySession {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let temporary_session = req
            .extensions()
            .get::<InertiaTemporarySession>()
            .cloned()
            .unwrap_or_default();

        std::future::ready(Ok(temporary_session))
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::config::InertiaConfig;
    use crate::inertia::{InertiaHttpRequest, InertiaResponder, ViewData};
    use crate::props::InertiaProp;
    use crate::providers::actix::headers::{
        InertiaHeader, X_INERTIA_PARTIAL_COMPONENT, X_INERTIA_PARTIAL_DATA,
        X_INERTIA_PARTIAL_EXCEPT,
    };
    use crate::req_type::PartialComponent;
    use crate::template_resolver::TemplateResolver;
    use crate::{
        hashmap, Component, Inertia, InertiaError, InertiaPage, InertiaTemporarySession,
        InertiaVersion, IntoInertiaPropResult,
    };
    use actix_web::body::MessageBody;
    use actix_web::http::header::HeaderValue;
    use actix_web::{test, HttpMessage, Responder};
    use serde_json::Value;

    use super::resolve_props;

    #[test]
    async fn test_get_partials_requirements() {
        let mut request = test::TestRequest::default();
        request = request.insert_header((X_INERTIA_PARTIAL_COMPONENT, "/Index"));
        request = request.insert_header((X_INERTIA_PARTIAL_DATA, "events,popularUsers")); // not any props but events and popularUsers
        request = request.insert_header((X_INERTIA_PARTIAL_EXCEPT, "auth")); // all props but auth

        let request = request.to_http_request();

        let partials = request.get_request_type().unwrap();

        assert_eq!(
            partials.unwrap_partial(),
            PartialComponent {
                only: vec!["events".to_string(), "popularUsers".to_string()],
                except: vec!["auth".to_string()],
                component: Component("/Index".to_string())
            }
        )
    }

    struct MyTemplateResolver;

    #[async_trait::async_trait(?Send)]
    impl TemplateResolver for MyTemplateResolver {
        async fn resolve_template(&self, view_data: ViewData<'_>) -> Result<String, InertiaError> {
            // import the layout root using your favourite engine
            // and renders it passing to it the view_data!
            let page = view_data.page;
            Ok(format!(
                "<div id='app' data-page='{}'><div>",
                serde_json::to_string(&page).unwrap()
            ))
        }
    }

    #[test]
    async fn test_inertia_page() {
        let inertia = Inertia::new(
            InertiaConfig::builder()
                .set_url("https://my-inertia-website.com")
                .set_version(InertiaVersion::Resolver(Box::new(|| "gen_the_version")))
                .set_template_resolver(Box::new(MyTemplateResolver))
                .build(),
        )
        .unwrap();

        let props = hashmap![
            "title" => InertiaProp::Data("My website's cool title!".into_inertia_value()),
            "content" => InertiaProp::Data("Such a nice content, isn't it?".into_inertia_value()),
        ];

        let fake_req = test::TestRequest::get()
            .insert_header(InertiaHeader::Inertia.convert())
            .insert_header(InertiaHeader::Version("gen_the_version").convert())
            .uri("/users")
            .append_header((
                actix_web::http::header::HOST,
                "https://my-inertia-website.com".to_string(),
            ))
            .to_http_request();

        // this is usually called by the Inertia rendering methods, so you are not allowed to access
        // the url and version! Let's mock it for this example, then!
        let page = InertiaPage::new(
            Component("/Users/Index".into()),
            "/users",
            Some("gen_the_version"),
            resolve_props(&props, &fake_req.get_request_type().unwrap())
                .await
                .unwrap(),
            None,
            None,
            false,
            false,
        );

        let body = inertia
            .inner_render(&fake_req, Component("/Users/Index".into()), Some(props))
            .await
            .unwrap()
            .into_body();

        let stringified_body = String::from_utf8(body.try_into_bytes().unwrap().to_vec()).unwrap();

        assert_eq!(
            Value::from_str(&stringified_body).unwrap(),
            serde_json::to_value(&page).unwrap(),
        );
    }

    #[tokio::test]
    async fn it_should_prefer_referer_header_over_session_prev_url() {
        let fake_req = test::TestRequest::get()
            .insert_header((
                actix_web::http::header::REFERER,
                HeaderValue::from_static("/foo"),
            ))
            .insert_header(InertiaHeader::Inertia.convert())
            .uri("/bar")
            .to_http_request();

        let inertia = Inertia::new(
            InertiaConfig::builder()
                .set_url("https://my-inertia-website.com")
                .set_version(InertiaVersion::Resolver(Box::new(|| "gen_the_version")))
                .set_template_resolver(Box::new(MyTemplateResolver))
                .build(),
        )
        .unwrap();

        let session = InertiaTemporarySession {
            errors: None,
            prev_req_url: "/".into(),
        };

        fake_req.extensions_mut().insert(session);

        let redirect_response = inertia.inner_back(&fake_req).respond_to(&fake_req);

        assert_eq!("/foo", redirect_response.headers().get("location").unwrap());
    }
}
