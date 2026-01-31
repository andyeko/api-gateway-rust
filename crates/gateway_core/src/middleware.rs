use crate::types::{Request, Response};

pub trait Middleware {
    fn process(&self, req: Request) -> Result<Request, Response>;
}

pub struct Logging;

impl Middleware for Logging {
    fn process(&self, req: Request) -> Result<Request, Response> {
        println!("[gateway] request path: {}", req.path);
        Ok(req)
    }
}

pub struct Auth;

impl Middleware for Auth {
    fn process(&self, mut req: Request) -> Result<Request, Response> {
        let has_key = req
            .headers
            .iter()
            .any(|(k, v)| k.eq_ignore_ascii_case("x-api-key") && !v.is_empty());

        if !has_key {
            return Err(Response::unauthorized("missing API key"));
        }

        req.headers.push(("x-auth".to_string(), "ok".to_string()));
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

pub fn default_pipeline() -> Vec<Box<dyn Middleware>> {
    vec![Box::new(Logging), Box::new(Auth), Box::new(HeaderInjection)]
}

pub fn apply(pipeline: &[Box<dyn Middleware>], req: Request) -> Result<Request, Response> {
    let mut current = req;
    for step in pipeline {
        current = step.process(current)?;
    }
    Ok(current)
}
