use super::*;
use core::fmt;
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub obj: String,
    pub creds: Creds,
}

/// Maybe more fields, like #topics subscribed to, active_since?
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub disp_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
/// Will have to add intermediate authorities, obviously.
pub enum Authority {
    Unknown,
    Subscriber,
    ServiceProvider,
    Admin,
}

impl From<&str> for Authority {
    // type Err = neo4rs::Error;
    fn from(s: &str) -> Self {
        match s {
            "Subscriber" => Self::Subscriber,
            "Admin" => Self::Admin,
            "Service" => Self::ServiceProvider,
            _ => Self::Unknown,
        }
    }
}

impl Display for Authority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Admin => write!(f, "Admin"),
            Self::ServiceProvider => write!(f, "Service"),
            Self::Subscriber => write!(f, "Subscriber"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub auth: Authority,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    status: String,
    message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

pub struct Identity {
    pub user_id: String,
    pub auth: models::Authority,
}

impl FromRequest for Identity {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let data = req.app_data::<web::Data<State>>().unwrap();

        let token = req
            .cookie("token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        if token.is_none() {
            let json_error = models::ErrorResponse {
            status: "fail".to_string(),
            message: "Roses are red,\n violets are blue,\n please enter your credentials,\n we'd love to have you.".to_string(),
        };
            return ready(Err(ErrorUnauthorized(json_error)));
        }

        let claims = match decode::<models::TokenClaims>(
            &token.unwrap(),
            &DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(c) => c.claims,
            Err(_) => {
                let json_error = models::ErrorResponse {
                    status: "fail".to_string(),
                    message: "I find your lack of faith...disturbing".to_string(),
                };
                return ready(Err(ErrorUnauthorized(json_error)));
            }
        };

        let user_id = claims.sub;
        let auth = claims.auth;
        req.extensions_mut().insert::<String>(user_id.clone());

        ready(Ok(Identity { user_id, auth }))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Creds {
    pub username: String,
    pub password: String,
    pub disp_name: String,
    pub auth: models::Authority,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Kernel {
    pub username: String,
    pub password: String
}