use super::headers;
use super::middleware::SharedProps;

use crate::inertia::{Inertia, InertiaHttpRequest, InertiaResponder, InertiaService, ViewData};
use crate::props::InertiaProp;
use crate::props::InertiaProps;
use crate::req_type::{InertiaRequestType, PartialComponent};
use crate::utils::convert_struct_to_stringified_json;
use crate::utils::{inertia_err_msg, request_page_render};
use crate::{Component, InertiaError, InertiaPage, InertiaTemporarySession};

use actix_web::body::BoxBody;
use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::http::header::HeaderName;
use actix_web::http::StatusCode;
use actix_web::{
    web, App, FromRequest, HttpMessage, HttpRequest, HttpResponse, HttpResponseBuilder, Responder,
    ResponseError,
};
use async_trait::async_trait;
use std::collections::HashMap;

impl Responder for InertiaPage {
    type Body = BoxBody;

    #[inline]
    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponseBuilder::new(StatusCode::OK)
            .append_header(headers::InertiaHeader::Inertia.convert())
            .body(BoxBody::new(
                convert_struct_to_stringified_json(self).unwrap(),
            ))
    }
}

#[async_trait(?Send)]
impl<T> InertiaResponder<HttpResponse, HttpRequest> for Inertia<T>
where
    T: 'static,
{
    #[inline]
    async fn render(
        &self,
        req: &HttpRequest,
        component: Component,
    ) -> Result<HttpResponse, InertiaError> {
        self.render_with_props(req, component, HashMap::new()).await
    }

    #[inline]
    async fn render_with_props(
        &self,
        req: &HttpRequest,
        component: Component,
        props: InertiaProps,
    ) -> Result<HttpResponse, InertiaError> {
        let url = req.uri().to_string();
        let req_type: InertiaRequestType = req.get_request_type()?;

        if let Err(forced_refresh) = self.check_and_handle_version_mismatch(req) {
            return Ok(forced_refresh);
        };

        let mut props = InertiaProp::resolve_props(&props, req_type.clone());

        if let Some(SharedProps(shared_props)) = req.extensions().get::<SharedProps>() {
            let shared_props = InertiaProp::resolve_props(shared_props, req_type);
            props.extend(shared_props);
        }

        let page = InertiaPage::new(component, url, Some(self.version.to_string()), props);

        // if it's an inertia request, returns an InertiaPage object
        if req.is_inertia_request() {
            return Ok(page.respond_to(req));
        }

        let mut ssr_page = None;

        if self.ssr_url.is_some() {
            match request_page_render(self.ssr_url.as_ref().unwrap(), page.clone()).await {
                Err(err) => {
                    log::warn!(
                        "{}",
                        inertia_err_msg(format!(
                            "Error on server-side rendering page {}. {}",
                            page.component.0, err
                        ))
                    );
                }
                Ok(page) => {
                    ssr_page = Some(page);
                }
            };
        }

        let view_data = ViewData {
            ssr_page,
            page,
            custom_props: self.custom_view_data.clone(),
        };

        let html = match (self.template_resolver)(
            self.template_path,
            view_data,
            self.template_resolver_data,
        )
        .await
        {
            Err(err) => return Err(err),
            Ok(html) => html,
        };

        return Ok(HttpResponseBuilder::new(StatusCode::OK)
            .insert_header(headers::InertiaHeader::Inertia.convert())
            .insert_header(actix_web::http::header::ContentType::html())
            .body(html)
            .respond_to(req));
    }

    #[inline]
    fn location(req: &HttpRequest, url: &str) -> HttpResponse {
        if !req.is_inertia_request() {
            return HttpResponse::Found()
                .append_header((actix_web::http::header::LOCATION, url))
                .finish();
        }

        HttpResponseBuilder::new(StatusCode::CONFLICT)
            .append_header(headers::InertiaHeader::InertiaLocation(url).convert())
            .finish()
    }
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

impl<TApp> InertiaService for App<TApp>
where
    TApp: ServiceFactory<
        ServiceRequest,
        Config = (),
        Error = actix_web::error::Error,
        InitError = (),
    >,
{
    fn inertia_route<T>(self, path: &str, component: &'static str) -> Self
    where
        T: 'static,
    {
        self.route(
            path,
            web::get().to(move |req: HttpRequest| async move {
                crate::actix::render::<T>(&req, component.into()).await
            }),
        )
    }
}

impl InertiaHttpRequest for HttpRequest {
    fn is_inertia_request(&self) -> bool {
        match self.headers().get(headers::X_INERTIA) {
            None => false,
            Some(v) => !v.is_empty(),
        }
    }

    fn get_request_type(&self) -> Result<InertiaRequestType, InertiaError> {
        let partial_comp = self.headers().get(headers::X_INERTIA_PARTIAL_COMPONENT);

        if partial_comp.is_none() {
            return Ok(InertiaRequestType::Standard);
        }

        let partial_comp = partial_comp.unwrap().to_str();

        if partial_comp.is_err() {
            return Err(InertiaError::SerializationError(format!(
                "Failed to serialize header {}",
                headers::X_INERTIA_PARTIAL_COMPONENT
            )));
        }

        let component = Component(partial_comp.unwrap().into());
        let only = extract_partials_headers_content(self, &headers::X_INERTIA_PARTIAL_DATA)?;
        let except = extract_partials_headers_content(self, &headers::X_INERTIA_PARTIAL_EXCEPT)?;

        let partials = PartialComponent {
            component,
            only,
            except,
        };

        Ok(InertiaRequestType::Partial(partials))
    }

    /// Checks if application assets version matches.
    /// If the request contains the inertia version header, it will be checked.
    /// Otherwise, it means it does not have outdated assets and can also pass.
    fn check_inertia_version(&self, current_version: &str) -> bool {
        self.headers()
            .get(headers::X_INERTIA_VERSION)
            .map_or(true, |version| {
                version
                    .to_str()
                    .map_or(false, |version| version == current_version)
            })
    }
}

fn extract_partials_headers_content(
    req: &HttpRequest,
    header_name: &HeaderName,
) -> Result<Vec<String>, InertiaError> {
    let partials = match req.headers().get(header_name) {
        None => Vec::new(),
        Some(value) => match value.to_str() {
            Ok(value) => value.split(",").map(|v| v.to_string()).collect(),
            Err(_err) => {
                return Err(InertiaError::HeaderError(format!(
                    "Header {}'s value must contain only printable ASCII characters.",
                    header_name,
                )))
            }
        },
    };

    Ok(partials)
}

pub trait InertiaActixHelpers {
    fn check_and_handle_version_mismatch(&self, req: &HttpRequest) -> Result<(), HttpResponse>;
}

impl<T> InertiaActixHelpers for Inertia<T>
where
    T: 'static,
{
    fn check_and_handle_version_mismatch(&self, req: &HttpRequest) -> Result<(), HttpResponse> {
        if req.is_inertia_request() && !req.check_inertia_version(self.version) {
            // tries to reflash Inertia session
            let inertia_session = req.extensions_mut().remove::<InertiaTemporarySession>();
            if let Err(err) = (self.reflash_inertia_session)(inertia_session) {
                log::warn!(
                    "{}",
                    inertia_err_msg(format!(
                        "Failed to reflesh Inertia Temporary Session. {}",
                        err.get_cause()
                    ))
                );
            };

            return Err(Self::location(req, &req.uri().to_string()));
        }

        Ok(())
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
    use crate::config::InertiaConfig;
    use crate::inertia::{InertiaHttpRequest, InertiaResponder, ViewData};
    use crate::props::InertiaProp;
    use crate::providers::actix::headers::{
        InertiaHeader, X_INERTIA_PARTIAL_COMPONENT, X_INERTIA_PARTIAL_DATA,
        X_INERTIA_PARTIAL_EXCEPT,
    };
    use crate::req_type::PartialComponent;
    use crate::{
        Component, Inertia, InertiaError, InertiaPage, InertiaVersion, TemplateResolverOutput,
    };
    use actix_web::body::MessageBody;
    use actix_web::test;
    use serde_json::json;
    use std::collections::HashMap;
    use std::str::from_utf8;

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

    #[test]
    async fn test_inertia_page() {
        async fn resolver(
            _path: &str,
            view_data: ViewData,
            _data: &'static (),
        ) -> Result<String, InertiaError> {
            // import the layout root using your favourite engine
            // and renders it passing to it the view_data!
            Ok(format!(
                "<div id='app' data-page='{}'><div>",
                serde_json::to_string(&view_data.page).unwrap()
            ))
        }

        fn resolver_wrapper(
            path: &'static str,
            view_data: ViewData,
            _data: &'static (),
        ) -> TemplateResolverOutput {
            Box::pin(resolver(path, view_data, _data))
        }

        let inertia = Inertia::new(
            InertiaConfig::builder()
                .set_url("https://my-inertia-website.com")
                .set_version(InertiaVersion::Resolver(Box::new(|| "gen_the_version")))
                .set_template_path("/resources/view/template.hbs")
                .set_template_resolver(&resolver_wrapper)
                .set_template_resolver_data(&())
                .build(),
        )
        .unwrap();

        let mut props: HashMap<String, InertiaProp> = HashMap::<String, InertiaProp>::new();
        props.insert(
            "title".into(),
            InertiaProp::Data("My website's cool title!".into()),
        );
        props.insert(
            "content".into(),
            InertiaProp::Data("Such a nice content, isn't it?".into()),
        );

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
            "/users".to_string(),
            Some("gen_the_version".to_string()),
            InertiaProp::resolve_props(&props, fake_req.get_request_type().unwrap()),
        );

        let body = inertia
            .render_with_props(&fake_req, Component("/Users/Index".into()), props)
            .await
            .unwrap()
            .into_body();

        assert_eq!(
            from_utf8(&body.try_into_bytes().unwrap()[..]).unwrap(),
            serde_json::to_string(&json!(page)).unwrap(),
        );
    }
}
