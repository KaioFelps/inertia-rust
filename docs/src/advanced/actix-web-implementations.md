# Actix Web Implementations

Check some useful snippets of middlewares mentioned by the previous chapter.

## Temporary Session Middleware (with Reflash)

This middleware uses actix sessions to manage the Inertia Rust temporary sessions. It's responsible
for:

* Before the route execution:
    * Removing flash data from user's session (errors, previous and current URLs);
    * Instantiating a `InertiaTemporarySession` and injecting it to the current request;
* After the route responds:
    * Check if there is a request for reflashing the current session (and reflashes it, if so);
    * Otherwise, adds the new "previous" and "current" requests URLs and the `SessionErrors` to the
      actual user's session.

```toml
# ./Cargo.toml
[dependencies]
serde = { version = "1.0.217", features = ["derive"]}
actix-web = "4.9.0"
actix-session = "0.10.1"
inertia-rust = { version = "2.0.0", features = ["actix"] }
log = "0.4.22"
serde_json = "1.0"
```

```rust
use actix_session::SessionExt;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use actix_web::HttpMessage;
use futures_util::future::LocalBoxFuture;
use inertia_rust::{InertiaSessionToReflash, InertiaTemporarySession, actix::SessionErrors};
use log::error;
use serde_json::Map;
use std::future::{ready, Ready};

pub struct ReflashTemporarySession;

impl<S, B> Transform<S, ServiceRequest> for ReflashTemporarySession
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ReflashTemporarySessionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ReflashTemporarySessionMiddleware { service }))
    }
}

pub struct ReflashTemporarySessionMiddleware<S> {
    service: S,
}

const ERRORS_KEY: &str = "_errors";
const PREV_REQ_KEY: &str = "_prev_req_url";
const CURR_REQ_KEY: &str = "_curr_req_url";

impl<S, B> Service<ServiceRequest> for ReflashTemporarySessionMiddleware<S>
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
        let session = req.get_session();

        let errors = session.remove(ERRORS_KEY).map(|errors| {
            serde_json::from_str(&errors).unwrap_or_else(|err| {
                error!("Failed to serialize session errors: {}", err);
                Map::new()
            })
        });

        let before_prev_url = session
            .get::<String>(PREV_REQ_KEY)
            .unwrap_or(None)
            .unwrap_or("/".into());

        let prev_url = session
            .get::<String>(CURR_REQ_KEY)
            .unwrap_or(None)
            .unwrap_or("/".into());

        // ---

        let temporary_session = InertiaTemporarySession {
            errors,
            prev_req_url: prev_url.clone(),
        };

        req.extensions_mut().insert(temporary_session);

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;

            let req = res.request();
            let session = req.get_session();

            let inertia_session = req.extensions_mut().remove::<InertiaSessionToReflash>();

            // if it needs to reflash a temporary flash session, then
            // replace data from inertia session middleware with the same as before,
            // so that the further request generates the same InertiaTemporarySession,
            // containing the exactly same errors, previous url, and current url.
            //
            // otherwise, gets the previous request's URI and stores the current one's as the next
            // request "previous", moving the navigation history
            let (prev_url, curr_url, optional_errors) =
                if let Some(InertiaSessionToReflash(inertia_session)) = inertia_session {
                    (before_prev_url, inertia_session.prev_req_url, inertia_session.errors)
                } else {
                    let errors = req
                        .extensions_mut()
                        .remove::<SessionErrors>()
                        .map(|SessionErrors(errors)| errors);

                    (prev_url, req.uri().to_string(), errors)
                };

            if let Some(errors) = optional_errors {
                if let Err(err) = session.insert(ERRORS_KEY, inertia_session.errors) {
                    error!("Failed to add errors to session: {}", err);
                }
            }

            if let Err(err) = session.insert(PREV_REQ_KEY, prev_url) {
                error!("Failed to update session previous request URL: {}", err);
            };

            if let Err(err) = session.insert(CURR_REQ_KEY, curr_url) {
                error!("Failed to update session current request URL: {}", err);
            };

            Ok(res)
        })
    }
}
```

Yet you need to enable your framework session middleware and manager (or your own). As errors are
retrieved by `remove` method, they are **only available for one request lifetime**. Indeed, errors
and flash messages shouldn't persist across multiple requests.

> Note: Be sure to register this middleware always after `InertiaMiddleware`. Since actix web calls
> the middlewares in the opposite order they've been registered, doing this will ensure that
> `InertiaMiddleware` has the correct `InertiaTemporarySession` when it's finally executed.

For more details on how to configure actix session, refer to their own documentation.
