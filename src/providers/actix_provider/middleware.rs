use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::{Method, StatusCode};
use actix_web::HttpMessage;
use actix_web::{Error, HttpRequest};
use futures::FutureExt;
use futures_util::future::LocalBoxFuture;
use std::collections::HashMap;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use crate::temporary_session::InertiaTemporarySession;
use crate::{InertiaProp, InertiaProps};

type SharedPropsCallback<'a> =
    Arc<dyn Fn(&HttpRequest) -> Pin<Box<dyn Future<Output = InertiaProps<'a>>>>>;

pub struct InertiaMiddleware<'a> {
    shared_props_cb: SharedPropsCallback<'a>,
}

impl Default for InertiaMiddleware<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> InertiaMiddleware<'a> {
    pub fn new() -> Self {
        Self {
            shared_props_cb: Arc::new(move |_req| async move { HashMap::new() }.boxed()),
        }
    }

    pub fn with_shared_props(mut self, props: SharedPropsCallback<'a>) -> Self {
        self.shared_props_cb = props;
        self
    }
}

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<'a, S, B> Transform<S, ServiceRequest> for InertiaMiddleware<'a>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'a,
    S::Future: 'static,
    B: 'static,
    'a: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = InertiaMiddlewareService<'a, S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let shpcb = self.shared_props_cb.clone();
        ready(Ok(InertiaMiddlewareService {
            service: Rc::new(service),
            shared_props: shpcb,
        }))
    }
}

pub struct InertiaMiddlewareService<'a, S> {
    service: Rc<S>,
    shared_props: SharedPropsCallback<'a>,
}

pub(crate) struct SharedProps<'a>(pub InertiaProps<'a>);

impl<'a, S, B> Service<ServiceRequest> for InertiaMiddlewareService<'a, S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'a,
    S::Future: 'static,
    B: 'static,
    'a: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srvc = self.service.clone();
        let shared_props = self.shared_props.clone();

        async move {
            let mut shared_props = shared_props(req.request()).await;

            if let Some(request_props) = req.extensions().get::<InertiaTemporarySession>() {
                shared_props.insert("errors", InertiaProp::always(&request_props.errors));
            }

            req.extensions_mut().insert(SharedProps(shared_props));

            let fut = srvc.call(req);

            let mut res = fut.await?;

            let req_method = res.request().method();
            let res_status = res.status();

            if [Method::PATCH, Method::PUT, Method::DELETE].contains(req_method)
                && (res_status == StatusCode::MOVED_PERMANENTLY || res_status == StatusCode::FOUND)
            {
                let res = res.response_mut();
                *res.status_mut() = StatusCode::SEE_OTHER;
            }

            Ok(res)
        }
        .boxed_local()
    }
}
