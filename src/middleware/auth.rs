use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;
use tracing::{info, error};

use crate::config::Config;
use crate::utils::jwt;

pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let span = tracing::span!(tracing::Level::INFO, "auth_middleware",
            path = %req.path(),
            method = %req.method());
        let _enter = span.enter();

        info!("Authorizing request");

        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let config = req.app_data::<actix_web::web::Data<Config>>().unwrap();

            // Отримуємо токен з Authorization header
            let auth_header = req.headers().get("Authorization");
            let auth_header = match auth_header {
                Some(header) => header.to_str().unwrap_or(""),
                None => {
                    error!("No authorization header found");
                    return Err(actix_web::error::ErrorUnauthorized("No authentication token provided"));
                }
            };

            // Перевіряємо формат Bearer token
            if !auth_header.starts_with("Bearer ") {
                error!("Invalid token format");
                return Err(actix_web::error::ErrorUnauthorized("Invalid token format"));
            }

            let token = &auth_header[7..]; // Видаляємо "Bearer " з початку

            // Валідуємо токен
            match jwt::validate_token(token, config) {
                Ok(claims) => {
                    let user_id = claims.sub.parse::<i32>().unwrap();
                    info!(
                        user_id = %user_id,
                        role = %claims.role,
                        "Token validation successful"
                    );

                    // Додаємо user_id в request extensions
                    req.extensions_mut().insert(user_id);
                    service.call(req).await
                }
                Err(e) => {
                    error!(error = %e, "Token validation failed");
                    Err(actix_web::error::ErrorUnauthorized("Invalid or expired token"))
                }
            }
        })
    }
}