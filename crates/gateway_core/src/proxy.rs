use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct Proxy {
    upstream_base: String,
    client: Client,
}

impl Proxy {
    pub fn new(upstream_base: impl Into<String>) -> Self {
        Self {
            upstream_base: upstream_base.into(),
            client: Client::new(),
        }
    }

    pub async fn forward(
        &self,
        req: Request<Body>,
        strip_prefix: &str,
    ) -> Result<Response<Body>, anyhow::Error> {
        let (parts, body) = req.into_parts();
        let body_bytes = body.collect().await?.to_bytes();

        let path_and_query = parts
            .uri
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/");

        let mut stripped = path_and_query
            .strip_prefix(strip_prefix)
            .unwrap_or(path_and_query)
            .to_string();

        if stripped.is_empty() {
            stripped = "/".to_string();
        }

        let target = format!("{}{}", self.upstream_base, stripped);

        let mut builder = self.client.request(parts.method, target);
        for (name, value) in parts.headers.iter() {
            builder = builder.header(name, value);
        }

        let upstream = builder.body(body_bytes).send().await?;
        let status = upstream.status();
        let headers = upstream.headers().clone();
        let upstream_body = upstream.bytes().await?;

        let mut response = Response::builder().status(status);
        for (name, value) in headers.iter() {
            response = response.header(name, value);
        }

        response
            .body(Body::from(upstream_body))
            .map_err(|err| anyhow::anyhow!("build response: {err}"))
    }
}

pub fn bad_gateway(message: impl Into<String>) -> Response<Body> {
    let body = Body::from(message.into());
    Response::builder()
        .status(StatusCode::BAD_GATEWAY)
        .body(body)
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::empty())
                .unwrap()
        })
}
