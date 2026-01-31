use std::sync::Arc;

use axum::body::Body;
use axum::extract::Extension;
use axum::http::{HeaderName, HeaderValue, Request, Response, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::any;
use axum::Router;

use crate::config::GatewayConfig;
use crate::middleware;
use crate::proxy::{bad_gateway, Proxy};
use crate::rate_limit::RateLimiter;
use crate::types::Request as GatewayRequest;

#[derive(Clone)]
struct GatewayState {
    limiter: Arc<RateLimiter>,
    pipeline: Arc<middleware::Pipeline>,
    proxy: Option<Proxy>,
}

pub async fn run(config: &GatewayConfig) -> anyhow::Result<()> {
    run_with_admin_router(config, None).await
}

pub async fn run_with_admin_router(
    config: &GatewayConfig,
    admin_router: Option<Router>,
) -> anyhow::Result<()> {
    println!("gateway listening on {}", config.listen_addr);
    println!("admin upstream {}", config.admin_upstream_base);

    let limiter = Arc::new(RateLimiter::new(100));
    let pipeline = Arc::new(middleware::default_pipeline());
    let proxy = if admin_router.is_some() {
        None
    } else {
        Some(Proxy::new(config.admin_upstream_base.clone()))
    };

    let state = Arc::new(GatewayState {
        limiter,
        pipeline,
        proxy,
    });

    let app = if let Some(admin_router) = admin_router {
        Router::new().nest("/admin", admin_router)
    } else {
        Router::new()
            .route("/admin", any(proxy_admin))
            .route("/admin/*path", any(proxy_admin))
    };

    let app = app
        .layer(axum::middleware::from_fn(gateway_checks))
        .layer(Extension(state.clone()));

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn gateway_checks(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Response<Body>> {
    let Some(state) = req.extensions().get::<Arc<GatewayState>>() else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "gateway state missing").into_response());
    };
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    if !state.limiter.allow(client_ip) {
        return Err((StatusCode::TOO_MANY_REQUESTS, "rate limited").into_response());
    }

    let mut gateway_req = GatewayRequest::new(req.uri().path());
    gateway_req.headers = req
        .headers()
        .iter()
        .filter_map(|(name, value)| value.to_str().ok().map(|v| (name.to_string(), v.to_string())))
        .collect();

    let updated = match middleware::apply(state.pipeline.as_ref(), gateway_req) {
        Ok(updated) => updated,
        Err(res) => {
            let status = StatusCode::from_u16(res.status).unwrap_or(StatusCode::UNAUTHORIZED);
            return Err((status, res.body).into_response());
        }
    };

    for (name, value) in updated.headers {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(&value),
        ) {
            req.headers_mut().insert(name, value);
        }
    }

    Ok(next.run(req).await)
}

async fn proxy_admin(
    Extension(state): Extension<Arc<GatewayState>>,
    req: Request<Body>,
) -> Response<Body> {
    let Some(proxy) = state.proxy.as_ref() else {
        return (StatusCode::BAD_GATEWAY, "admin proxy not configured").into_response();
    };

    match proxy.forward(req, "/admin").await {
        Ok(response) => response,
        Err(err) => bad_gateway(format!("upstream error: {err}")),
    }
}
