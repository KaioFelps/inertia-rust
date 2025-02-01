//! See: https://gist.github.com/KaioFelps/a5308ab73f8fa22d268240958ddbd1cb

use actix_web::dev::{self, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use rand::Rng;
use std::future::{ready, Ready};

// the file_session.rs file from this GIST
use crate::config::file_session::clean_expired_sessions;

// Just like Laravel, this is used to raffle a request to run the
// garbage collector, which will remove unused resources
// (such as expired sessions files)
const LOTTERY: [u8; 2] = [2, 100];

pub struct GarbageCollectorMiddleware;

impl GarbageCollectorMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for GarbageCollectorMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = GarbageCollectorMiddlewareTransform<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(GarbageCollectorMiddlewareTransform { service }))
    }
}

pub struct GarbageCollectorMiddlewareTransform<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for GarbageCollectorMiddlewareTransform<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let lottery_num = rand::rng().random_range(0..LOTTERY[1]);
        let is_raffled = lottery_num <= LOTTERY[0];

        let fut = self.service.call(request);

        Box::pin(async move {
            let res = fut.await?;

            if is_raffled {
                println!("Request raffled to run the garbage collector.");
                let _ = clean_expired_sessions().await;
            }

            Ok(res)
        })
    }
}
