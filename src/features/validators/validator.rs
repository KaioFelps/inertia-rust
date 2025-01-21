use serde::de::DeserializeOwned;
use validator::Validate;

pub trait InertiaValidateOrRedirect<T, TRequest, TRedirect>
where
    T: DeserializeOwned + Validate,
{
    fn validate_or_back(self, req: &TRequest) -> Result<T, TRedirect>;
}

#[cfg(feature = "actix-validator")]
pub mod actix_validator {
    use std::collections::HashMap;

    use actix_web::{
        web::{self, Redirect},
        HttpRequest,
    };
    use serde::de::DeserializeOwned;
    use validator::Validate;

    use crate::{Inertia, InertiaFacade};

    use super::InertiaValidateOrRedirect;

    impl<T: DeserializeOwned + Validate> InertiaValidateOrRedirect<T, HttpRequest, Redirect> for T {
        #[inline]
        fn validate_or_back(self, req: &HttpRequest) -> Result<T, Redirect> {
            match self.validate() {
                Ok(_) => Ok(self),
                Err(errors) => {
                    let errors = errors
                        .field_errors()
                        .into_iter()
                        .map(|(key, v)| {
                            (
                                key,
                                v[0].message.as_ref().unwrap_or(&v[0].code).clone().into(),
                            )
                        })
                        .collect::<HashMap<_, _>>();

                    Err(Inertia::back_with_errors(req, errors))
                }
            }
        }
    }

    impl<T: DeserializeOwned + Validate> InertiaValidateOrRedirect<T, HttpRequest, Redirect>
        for web::Json<T>
    {
        #[inline]
        fn validate_or_back(self, req: &HttpRequest) -> Result<T, Redirect> {
            self.into_inner().validate_or_back(req)
        }
    }
}
