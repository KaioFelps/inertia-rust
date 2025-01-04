use super::impls::ShallEncryptHistory;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use actix_web::HttpMessage;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

pub struct EncryptHistoryMiddleware;

impl Default for EncryptHistoryMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl EncryptHistoryMiddleware {
    pub fn new() -> Self {
        Self
    }
}

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for EncryptHistoryMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = EncryptHistoryMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(EncryptHistoryMiddlewareService { service }))
    }
}

pub struct EncryptHistoryMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for EncryptHistoryMiddlewareService<S>
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
        req.extensions_mut().insert(ShallEncryptHistory(true));

        let fut: <S as Service<ServiceRequest>>::Future = self.service.call(req);

        Box::pin(async move {
            let res: ServiceResponse<B> = fut.await?;
            Ok(res)
        })
    }
}
