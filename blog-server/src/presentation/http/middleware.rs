use crate::infra::jwt::JwtService;
use actix_web::dev::{Payload, ServiceRequest};
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::future::{Ready, ready};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let user = req.extensions().get::<AuthenticatedUser>().cloned();
        ready(user.ok_or_else(|| ErrorUnauthorized("authentication required")))
    }
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt = req.app_data::<web::Data<JwtService>>();
    let Some(jwt) = jwt else {
        tracing::error!("JWTService is missing from app_data");
        return Err((ErrorInternalServerError("server misconfigured"), req));
    };

    match jwt.verify_token(credentials.token()) {
        Ok(claims) => {
            req.extensions_mut().insert(AuthenticatedUser {
                user_id: claims.user_id,
                username: claims.username,
            });
            Ok(req)
        }
        Err(err) => {
            tracing::warn!(error = %err, "rejected request with invalid token");
            Err((ErrorUnauthorized("invalid or expired token"), req))
        }
    }
}
