use crate::inertia::InertiaResponder;
use crate::utils::inertia_err_msg;
use crate::{Component, Inertia, InertiaError, InertiaProps};
use actix_web::web::Data;
use actix_web::{HttpRequest, HttpResponse};

/// Short for calling `render` from the `Inertia` instance configured and added to the request
/// AppData.
///
/// # Arguments
/// * `req`         -   A reference to the HttpRequest.
/// * `component`   -   The name of the page javascript component.
///
/// # Panic
/// Panics if Inertia instance hasn't been configured (set to AppData).
pub async fn render<T>(
    req: &HttpRequest,
    component: Component,
) -> Result<HttpResponse, InertiaError>
where
    T: 'static,
{
    let inertia = extract_inertia::<T>(req);
    inertia.render(req, component).await
}

/// Short for calling `render_with_props` from the `Inertia` instance configured and added to the request
/// AppData.
///
/// # Arguments
/// * `req`         -   A reference to the HttpRequest.
/// * `component`   -   The name of the page javascript component.
///
/// # Panic
/// Panics if Inertia instance hasn't been configured (set to AppData).
pub async fn render_with_props<T>(
    req: &HttpRequest,
    component: Component,
    props: InertiaProps,
) -> Result<HttpResponse, InertiaError>
where
    T: 'static,
{
    let inertia: &Inertia<T> = extract_inertia(req);
    inertia.render_with_props(req, component, props).await
}

fn extract_inertia<T>(req: &HttpRequest) -> &Inertia<T>
where
    T: 'static,
{
    match req.app_data::<Data<Inertia<T>>>() {
        None => panic!("{}", &inertia_err_msg("There is no Inertia struct in AppData. Please, assure you have correctly configured Inertia.".into())),
        Some(inertia) => inertia
    }
}
