use std::collections::HashMap;
use actix_web::{HttpRequest, HttpResponse, HttpResponseBuilder, Responder};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use async_trait::async_trait;
use crate::{Component, InertiaError, InertiaPage};
use crate::utils::{inertia_err_msg, request_page_render};
use crate::inertia::{Inertia, InertiaHttpRequest, InertiaResponder, ViewData};
use crate::props::InertiaProp;
use crate::props::InertiaProps;
use crate::req_type::{InertiaRequestType, PartialComponent};
use crate::utils::convert_struct_to_stringified_json;

mod header_names {
    use actix_web::http::header::HeaderName;
    use crate::inertia;

    #[allow(unused)] pub const X_INERTIA: HeaderName = HeaderName::from_static(inertia::X_INERTIA);
    #[allow(unused)] pub const X_INERTIA_LOCATION: HeaderName = HeaderName::from_static(inertia::X_INERTIA_LOCATION);
    #[allow(unused)] pub const X_INERTIA_VERSION: HeaderName = HeaderName::from_static(inertia::X_INERTIA_VERSION);
    #[allow(unused)] pub const X_INERTIA_PARTIAL_COMPONENT: HeaderName = HeaderName::from_static(inertia::X_INERTIA_PARTIAL_COMPONENT);
    #[allow(unused)] pub const X_INERTIA_PARTIAL_DATA: HeaderName = HeaderName::from_static(inertia::X_INERTIA_PARTIAL_DATA);
    #[allow(unused)] pub const X_INERTIA_PARTIAL_EXCEPT: HeaderName = HeaderName::from_static(inertia::X_INERTIA_PARTIAL_EXCEPT);
}

pub enum InertiaHeader {
    Inertia,
    InertiaLocation(String),
    InertiaPartialData(Vec<String>)
}

impl InertiaHeader {
    pub fn convert(&self) -> (HeaderName, HeaderValue) {
        match self {
            Self::Inertia => (header_names::X_INERTIA, HeaderValue::from_str("true").unwrap()),
            Self::InertiaLocation(path) => (header_names::X_INERTIA_LOCATION, HeaderValue::from_str(path.as_str()).unwrap()),
            Self::InertiaPartialData(partials) => {
                if partials.len() < 1 {
                    return (header_names::X_INERTIA_PARTIAL_DATA, HeaderValue::from_str("").unwrap());
                }

                let mut str_partials: String = String::from(&partials[0]);

                for part in partials[1..].iter() {
                    str_partials.push_str(&",");
                    str_partials.push_str(part);
                }

                (header_names::X_INERTIA_PARTIAL_DATA, HeaderValue::from_str(&str_partials.as_str()).unwrap())
            }
        }
    }
}

impl<'response_lt> Responder for InertiaPage<'response_lt> {
    type Body = BoxBody;

    #[inline]
    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let mut builder = HttpResponseBuilder::new(StatusCode::OK);
        builder.append_header(InertiaHeader::Inertia.convert());

        let builder = builder.body(BoxBody::new(convert_struct_to_stringified_json(self).unwrap()));
        builder
    }
}

#[async_trait(?Send)]
impl InertiaResponder<HttpResponse, HttpRequest> for Inertia {
    #[inline]
    async fn render(&self, req: &HttpRequest, component: Component) -> Result<HttpResponse, InertiaError> {
        self.render_with_props(&req, component, HashMap::new()).await
    }

    #[inline]
    async fn render_with_props(
        &self,
        req: &HttpRequest,
        component: Component,
        props: InertiaProps
    ) -> Result<HttpResponse, InertiaError>  {
        let url = req.uri().to_string();
        let req_type = req.get_request_type()?;
        let props = InertiaProp::resolve_props(props, req_type);

        let page = InertiaPage::new(
            component,
            url,
            Some(self.version),
            props,
        );

        // if it's an inertia request, returns an InertiaPage object
        if req.is_inertia_request() {
            return Ok(page.respond_to(&req));
        }

        let mut ssr_page = None;

        if self.ssr_url.is_some() {
            match request_page_render(&self.ssr_url.as_ref().unwrap(), page.clone()).await {
                Err(err) => {
                    log::warn!("{}", inertia_err_msg(format!(
                        "Failed to server-side render the page: {:#?}",
                        err
                    )));
                },
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

        let html = (self.template_resolver)(self.template_path, view_data).await;

        if html.is_err() {
            if let InertiaError::SsrError(err) = html.unwrap_err() {
                return Err(InertiaError::SsrError(err));
            }

            let internal_err = inertia_err_msg("Unexpected server-side rendering error.".into());

            return Ok(HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR)
                .body(internal_err)
                .respond_to(req));
        }

        let html = html.unwrap();
        return Ok(HttpResponseBuilder::new(StatusCode::OK)
            .insert_header(InertiaHeader::Inertia.convert())
            .insert_header(actix_web::http::header::ContentType::html())
            .body(html)
            .respond_to(req));
    }

    #[inline]
    fn redirect(&self, location: String) -> HttpResponse {
        let mut builder = HttpResponseBuilder::new(StatusCode::CONFLICT);
        builder.append_header(InertiaHeader::InertiaLocation(location).convert());
        return builder.finish();
    }
}

impl InertiaHttpRequest for HttpRequest {
    fn is_inertia_request(&self) -> bool {
        return match self.headers().get(header_names::X_INERTIA) {
            None => false,
            Some(v) => !v.is_empty()
        }
    }

    fn get_request_type(&self) -> Result<InertiaRequestType, InertiaError> {
        let partial_comp = self.headers().get(header_names::X_INERTIA_PARTIAL_COMPONENT);

        if partial_comp.is_none() {
            return Ok(InertiaRequestType::Standard);
        }

        let partial_comp = partial_comp.unwrap().to_str();

        if partial_comp.is_err() {
            return Err(InertiaError::SerializationError(
                format!("Failed to serialize header {}", header_names::X_INERTIA_PARTIAL_COMPONENT.to_string())
            ));
        }

        let component = Component(partial_comp.unwrap().into());
        let only = extract_partials_headers_content(self, &header_names::X_INERTIA_PARTIAL_DATA)?;
        let except = extract_partials_headers_content(self, &header_names::X_INERTIA_PARTIAL_EXCEPT)?;

        let partials = PartialComponent {
            component,
            only,
            except
        };

        return Ok(InertiaRequestType::Partial(partials));
    }
}


fn extract_partials_headers_content(req: &HttpRequest, header_name: &HeaderName) -> Result<Vec<String>, InertiaError> {
    let partials = match req.headers().get(header_name) {
        None => Vec::new(),
        Some(value) => {
            let value = value.to_str();

            if value.is_err() {
                return Err(InertiaError::HeaderError(
                    format!(
                        "Header {}'s value must contain only printable ASCII characters.",
                        header_name.to_string(),
                    )
                ))
            };

            let value = value.unwrap()
                .split(",")
                .map(|v| v.to_string())
                .collect();

            value
        }
    };

    return Ok(partials);
}

#[cfg(test)]
mod test {
    use crate::providers::actix::header_names::{
        X_INERTIA_PARTIAL_COMPONENT,
        X_INERTIA_PARTIAL_DATA,
        X_INERTIA_PARTIAL_EXCEPT
    };
    use std::collections::HashMap;
    use std::future::Future;
    use std::pin::Pin;
    use std::str::from_utf8;
    use crate::{InertiaPage, Inertia, Component, InertiaVersion, InertiaError};
    use actix_web::test;
    use actix_web::body::MessageBody;
    use serde_json::json;
    use crate::providers::actix::InertiaHeader;
    use crate::inertia::{InertiaHttpRequest, InertiaResponder, ViewData};
    use crate::props::InertiaProp;
    use crate::req_type::PartialComponent;

    #[test]
    async fn test_get_partials_requirements() {
        let mut request = actix_web::test::TestRequest::default();
        request = request.insert_header((X_INERTIA_PARTIAL_COMPONENT, "/Index"));
        request = request.insert_header((X_INERTIA_PARTIAL_DATA, "events,popularUsers")); // not any props but events and popularUsers

        request = request.insert_header((X_INERTIA_PARTIAL_EXCEPT, "auth")); // all props but auth
        let request = request.to_http_request();

        let partials = request.get_request_type().unwrap();

        assert_eq!(partials.unwrap_partial(), PartialComponent {
            only: vec!["events".to_string(), "popularUsers".to_string()],
            except: vec!["auth".to_string()],
            component: Component("/Index".to_string())
        })
    }

    #[test]
    async fn test_inertia_page() {
        fn resolver(
            path: &str,
            view_data: ViewData
        ) -> Pin<Box<dyn Future<Output = Result<String, InertiaError>> + Send + 'static>> {
            return Box::pin(async move {
                // import the layout root using your favourite engine
                // and renders it passing to it the view_data!
                Ok("<h1>my rendered page!</h1>".to_string())
            });
        }

        let inertia = Inertia::new(
            "https://my-inertia-website.com",                           // url
            InertiaVersion::Resolver(|| "gen_the_version".to_string()),     // (assets) version
            "/resources/view/template.hbs",                     // template path
            resolver,                                                       // the template resolver
        );

        let mut props = HashMap::<String, InertiaProp>::new();
        props.insert("title".into(), InertiaProp::Data("My website's cool title!".into()));
        props.insert("content".into(), InertiaProp::Data("Such a nice content, isn't it?".into()));

        let props_clone = props.clone();

        let fake_req = actix_web::test::TestRequest::get();
        let fake_req = fake_req.insert_header(InertiaHeader::Inertia.convert());
        let fake_req = fake_req.uri("/users");
        let fake_req = fake_req.append_header((actix_web::http::header::HOST, "https://my-inertia-website.com".to_string()));
        let fake_req = fake_req.to_http_request();

        // this is usually called by the Inertia rendering methods, so you are not allowed to access
        // the url and version! Let's mock it for this example, then!
        let page = InertiaPage::new(
            Component("/Users/Index".into()),
            "/users".to_string(),
            Some("gen_the_version"),
            InertiaProp::resolve_props(props_clone, fake_req.get_request_type().unwrap())
        );

        let rendered_resp = inertia.render_with_props(&fake_req, Component("/Users/Index".into()), props);
        let body = rendered_resp.await.unwrap().into_body();

        assert_eq!(
            from_utf8(&body.try_into_bytes().unwrap()[..]).unwrap(),
            serde_json::to_string(&json!(page)).unwrap(),
        );
    }
}
