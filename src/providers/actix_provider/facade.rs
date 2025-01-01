use crate::facade::InertiaFacade;
use crate::inertia::InertiaResponder;
use crate::utils::inertia_err_msg;
use crate::{Component, Inertia, InertiaError, InertiaProps};
use actix_web::web::Data;
use actix_web::{HttpRequest, HttpResponse};
use async_trait::async_trait;

#[async_trait(?Send)]
impl InertiaFacade<HttpRequest, HttpResponse> for Inertia {
    async fn render(req: &HttpRequest, component: Component) -> Result<HttpResponse, InertiaError> {
        let inertia = extract_inertia(req);
        inertia.inner_render(req, component).await
    }

    async fn render_with_props(
        req: &HttpRequest,
        component: Component,
        props: InertiaProps<'_>,
    ) -> Result<HttpResponse, InertiaError> {
        let inertia: &Inertia = extract_inertia(req);
        inertia.inner_render_with_props(req, component, props).await
    }

    fn location(req: &HttpRequest, url: &str) -> HttpResponse {
        <Inertia as InertiaResponder<HttpResponse, HttpRequest>>::inner_location(req, url)
    }
}

fn extract_inertia(req: &HttpRequest) -> &Inertia {
    match req.app_data::<Data<Inertia>>() {
        None => panic!("{}", &inertia_err_msg("There is no Inertia struct in AppData. Please, assure you have correctly configured Inertia.".into())),
        Some(inertia) => inertia
    }
}
