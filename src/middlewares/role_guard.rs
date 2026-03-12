use crate::services::auth_service::AccessClaims;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;

pub struct RoleGuard {
    allowed_roles: Vec<String>,
}

impl RoleGuard {
    pub fn require(role: &str) -> Self {
        Self {
            allowed_roles: vec![role.to_string()],
        }
    }

    pub fn any(roles: Vec<&str>) -> Self {
        Self {
            allowed_roles: roles.into_iter().map(String::from).collect(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RoleGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RoleGuardService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RoleGuardService {
            service: Rc::new(service),
            allowed_roles: self.allowed_roles.clone(),
        }))
    }
}

pub struct RoleGuardService<S> {
    service: Rc<S>,
    allowed_roles: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for RoleGuardService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let allowed_roles = self.allowed_roles.clone();

        Box::pin(async move {
            let claims = req.extensions().get::<AccessClaims>().cloned();

            match claims {
                None => Err(actix_web::error::ErrorUnauthorized(
                    r#"{"code":"UNAUTHORIZED","message":"Not authenticated"}"#,
                )),
                Some(c) if allowed_roles.contains(&c.role) => service.call(req).await,
                Some(c) => {
                    tracing::warn!(
                        user_id = %c.sub,
                        user_role = %c.role,
                        required_roles = ?allowed_roles,
                        "Access denied: insufficient role"
                    );
                    Err(actix_web::error::ErrorForbidden(
                        r#"{"code":"FORBIDDEN","message":"Insufficient permissions"}"#,
                    ))
                }
            }
        })
    }
}
