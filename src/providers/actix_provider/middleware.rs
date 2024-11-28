use actix_web::body::EitherBody;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::{Method, StatusCode};
use actix_web::web::Data;
use actix_web::Error;
use actix_web::HttpMessage;
use futures_util::future::LocalBoxFuture;
use serde_json::to_value;
use std::collections::HashMap;
use std::future::{ready, Ready};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::inertia::{InertiaHttpRequest, InertiaResponder};
use crate::temporary_session::InertiaTemporarySession;
use crate::{Inertia, InertiaProp, InertiaProps};

type SharedPropsCallback = dyn Fn(&ServiceRequest) -> InertiaProps;

pub struct InertiaMiddleware<TInertia> {
    shared_props_cb: Arc<SharedPropsCallback>,
    _p: PhantomData<TInertia>,
}

impl<TInertia> Default for InertiaMiddleware<TInertia>
where
    TInertia: 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<TInertia> InertiaMiddleware<TInertia>
where
    TInertia: 'static,
{
    pub fn new() -> Self {
        Self {
            shared_props_cb: Arc::new(|_req| HashMap::new()),
            _p: PhantomData::<TInertia>,
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
impl<S, B, TInertia> Transform<S, ServiceRequest> for InertiaMiddleware<TInertia>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    TInertia: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = InertiaMiddlewareService<S, TInertia>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let shpcb = self.shared_props_cb.clone();
        ready(Ok(InertiaMiddlewareService {
            service,
            shared_props: shpcb,
            _p: PhantomData::<TInertia>,
        }))
    }
}

pub struct InertiaMiddlewareService<S, TInertia>
where
    TInertia: 'static,
{
    service: S,
    shared_props: Arc<SharedPropsCallback>,
    _p: PhantomData<TInertia>,
}

pub(crate) struct SharedProps(pub InertiaProps);

impl<S, B, TInertia> Service<ServiceRequest> for InertiaMiddlewareService<S, TInertia>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    TInertia: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut shared_props = (self.shared_props)(&req);

        let inertia_temporary_session = req.extensions_mut().remove::<InertiaTemporarySession>();
        if let Some(request_props) = &inertia_temporary_session {
            let errors = to_value(&request_props.errors).unwrap();
            shared_props.insert("errors".into(), InertiaProp::Always(errors));
        }

        req.extensions_mut().insert(SharedProps(shared_props));

        let http_req = req.request().clone();
        let inertia = req.app_data::<Data<Inertia<TInertia>>>().cloned();

        let fut: <S as Service<ServiceRequest>>::Future = self.service.call(req);

        Box::pin(async move {
            // check inertia version and force refresh persisting temporary session
            let is_latest_version = inertia.map_or(false, |inertia| {
                http_req.check_inertia_version(inertia.version)
            });

            match is_latest_version {
                false => {
                    let response =
                        Inertia::<TInertia>::location(&http_req, &http_req.uri().to_string());

                    if let Some(session) = inertia_temporary_session {
                        http_req.extensions_mut().insert(session);
                    };

                    let res = ServiceResponse::new(http_req, response).map_into_right_body();

                    Ok(res)
                }
                true => {
                    let mut res = fut.await.map(ServiceResponse::map_into_left_body)?;

                    let req_method = res.request().method();
                    let res_status = res.status();

                    if [Method::PATCH, Method::PUT, Method::DELETE].contains(req_method)
                        && (res_status == StatusCode::MOVED_PERMANENTLY
                            || res_status == StatusCode::FOUND)
                    {
                        let res = res.response_mut();
                        *res.status_mut() = StatusCode::SEE_OTHER;
                    }

                    Ok(res)
                }
            }
        })
    }
}
