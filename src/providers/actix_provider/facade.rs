use std::collections::HashMap;

use crate::facade::InertiaFacade;
use crate::inertia::InertiaResponder;
use crate::utils::inertia_err_msg;
use crate::{hashmap, Component, Inertia, InertiaError, InertiaProps};
use actix_web::web::{Data, Redirect};
use actix_web::{HttpRequest, HttpResponse};
use async_trait::async_trait;
use serde_json::Value;

#[async_trait(?Send)]
impl InertiaFacade<HttpRequest, HttpResponse, Redirect> for Inertia {
    #[inline]
    async fn render(req: &HttpRequest, component: Component) -> Result<HttpResponse, InertiaError> {
        let inertia = extract_inertia(req);
        inertia.inner_render(req, component).await
    }

    #[inline]
    async fn render_with_props(
        req: &HttpRequest,
        component: Component,
        props: InertiaProps<'_>,
    ) -> Result<HttpResponse, InertiaError> {
        let inertia: &Inertia = extract_inertia(req);
        inertia.inner_render_with_props(req, component, props).await
    }

    #[inline]
    fn location(req: &HttpRequest, url: &str) -> HttpResponse {
        Inertia::inner_location(req, url)
    }

    #[inline]
    fn encrypt_history(req: &HttpRequest, encrypt: bool) {
        Inertia::inner_encrypt_history(req, encrypt);
    }

    #[inline]
    fn clear_history(req: &HttpRequest) {
        Inertia::inner_clear_history(req);
    }

    #[inline]
    fn back(req: &HttpRequest) -> Redirect {
        let inertia = extract_inertia(req);
        inertia.inner_back_with_errors(req, hashmap![])
    }

    #[inline]
    fn back_with_errors(req: &HttpRequest, errors: HashMap<&str, Value>) -> Redirect {
        let inertia = extract_inertia(req);
        inertia.inner_back_with_errors(req, errors)
    }
}

fn extract_inertia(req: &HttpRequest) -> &Inertia {
    match req.app_data::<Data<Inertia>>() {
        None => panic!("{}", &inertia_err_msg("There is no Inertia struct in AppData. Please, assure you have correctly configured Inertia.".into())),
        Some(inertia) => inertia
    }
}
