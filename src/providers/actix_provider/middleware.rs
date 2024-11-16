use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use actix_web::HttpMessage;
use futures_util::future::LocalBoxFuture;
use serde_json::to_value;
use std::collections::HashMap;
use std::future::{ready, Ready};
use std::sync::Arc;

use crate::temporary_messages::InertiaTemporarySession;
use crate::{InertiaProp, InertiaProps};

type SharedPropsCallback = dyn Fn(&ServiceRequest) -> InertiaProps;

pub struct InertiaMiddleware {
    shared_props_cb: Arc<SharedPropsCallback>,
}

impl Default for InertiaMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl InertiaMiddleware {
    pub fn new() -> Self {
        Self {
            shared_props_cb: Arc::new(|_req| HashMap::new()),
        }
    }

    pub fn with_shared_props(mut self, props: Arc<SharedPropsCallback>) -> Self {
        self.shared_props_cb = props;
        self
    }
}

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for InertiaMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = InertiaMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let shpcb = self.shared_props_cb.clone();
        ready(Ok(InertiaMiddlewareService {
            service,
            shared_props: shpcb,
        }))
    }
}

pub struct InertiaMiddlewareService<S> {
    service: S,
    shared_props: Arc<SharedPropsCallback>,
}

pub(crate) struct SharedProps(pub InertiaProps);

impl<S, B> Service<ServiceRequest> for InertiaMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut shared_props = (self.shared_props)(&req);

        if let Some(request_props) = req.extensions().get::<InertiaTemporarySession>() {
            let errors = to_value(&request_props.errors).unwrap();
            shared_props.insert("errors".into(), InertiaProp::Always(errors));
        }

        req.extensions_mut().insert(SharedProps(shared_props));

        let fut: <S as Service<ServiceRequest>>::Future = self.service.call(req);

        Box::pin(async move {
            let res: ServiceResponse<B> = fut.await?;
            Ok(res)
        })
    }
}
