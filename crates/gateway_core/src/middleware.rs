use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::types::{Request, Response};

/// JWT Claims structure (must match auth_core Claims)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub exp: u64,
    pub iat: u64,
    pub email: String,
    pub name: String,
    pub role: String,
    pub org_id: Option<String>,
}

pub trait Middleware: Send + Sync {
    fn process(&self, req: Request) -> Result<Request, Response>;
}

pub struct Logging;

impl Middleware for Logging {
    fn process(&self, req: Request) -> Result<Request, Response> {
        println!("[gateway] request path: {}", req.path);
        Ok(req)
    }
}

/// JWT secret for token validation (should match auth_core)
static JWT_SECRET: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub fn set_jwt_secret(secret: String) {
    let _ = JWT_SECRET.set(secret);
}

fn get_jwt_secret() -> &'static str {
    JWT_SECRET.get().map(|s| s.as_str()).unwrap_or("secret")
}

pub struct Auth;

impl Middleware for Auth {
    fn process(&self, mut req: Request) -> Result<Request, Response> {
        // Skip API key check for auth routes (login, register, etc.)
        if req.path.starts_with("/auth") {
            return Ok(req);
        }

        // Try to extract JWT token from Authorization header
        let auth_header = req
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
            .map(|(_, v)| v.clone());

        if let Some(auth) = auth_header {
            if let Some(token) = auth.strip_prefix("Bearer ").or_else(|| auth.strip_prefix("bearer ")) {
                // Decode JWT and extract claims
                let mut validation = Validation::new(Algorithm::HS256);
                validation.validate_exp = true;
                
                if let Ok(token_data) = decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(get_jwt_secret().as_bytes()),
                    &validation,
                ) {
                    let claims = token_data.claims;
                    
                    // Add user info headers
                    req.headers.push(("x-user-id".to_string(), claims.sub));
                    req.headers.push(("x-user-email".to_string(), claims.email));
                    req.headers.push(("x-user-name".to_string(), claims.name));
                    req.headers.push(("x-user-role".to_string(), claims.role));
                    
                    if let Some(org_id) = claims.org_id {
                        req.headers.push(("x-organisation-id".to_string(), org_id));
                    }
                    
                    req.headers.push(("x-auth".to_string(), "jwt".to_string()));
                    return Ok(req);
                }
            }
        }

        // Fallback: check for API key
        let has_key = req
            .headers
            .iter()
            .any(|(k, v)| k.eq_ignore_ascii_case("x-api-key") && !v.is_empty());

        if !has_key {
            return Err(Response::unauthorized("missing authentication"));
        }

        req.headers.push(("x-auth".to_string(), "api-key".to_string()));
        Ok(req)
    }
}

pub struct HeaderInjection;

impl Middleware for HeaderInjection {
    fn process(&self, mut req: Request) -> Result<Request, Response> {
        req.headers
            .push(("x-gateway".to_string(), "apisentinel".to_string()));
        Ok(req)
    }
}

pub type Pipeline = Vec<Box<dyn Middleware>>;

pub fn default_pipeline() -> Pipeline {
    vec![Box::new(Logging), Box::new(Auth), Box::new(HeaderInjection)]
}

pub fn apply(pipeline: &Pipeline, req: Request) -> Result<Request, Response> {
    let mut current = req;
    for step in pipeline {
        current = step.process(current)?;
    }
    Ok(current)
}
