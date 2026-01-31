use crate::types::{Request, Response};

#[derive(Debug, Clone)]
pub struct Proxy {
    upstream_base: String,
}

impl Proxy {
    pub fn new(upstream_base: impl Into<String>) -> Self {
        Self {
            upstream_base: upstream_base.into(),
        }
    }

    pub fn forward(&self, req: Request) -> Response {
        let target = format!("{}{}", self.upstream_base, req.path);
        Response::ok(format!("proxied to {target}"))
    }
}
