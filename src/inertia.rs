use serde::Serialize;

pub struct Inertia;

impl Inertia {
    /// Renders an Inertia Page as a Responder.
    pub fn render(path: String) {

    }

    /// Renders an Inertia Page with props as a Responder.
    ///
    /// ## Errors
    /// This operation may result in one of InertiaErrors if the props struct
    /// or any of its fields don't implement [`Serialize`] trait.
    ///
    /// [`Serialize`]: serde::Serialize
    pub fn render_with_props<TProps>(path: String, props: TProps) where TProps: Serialize {

    }
}
