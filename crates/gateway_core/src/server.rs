use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use axum::Router;
use axum::body::Body;
use axum::extract::Extension;
use axum::http::{HeaderName, HeaderValue, Request, Response, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::any;

use crate::config::{GatewayConfig, RouteMode};
use crate::middleware;
use crate::proxy::{Proxy, bad_gateway};
use crate::rate_limit::RateLimiter;
use crate::types::Request as GatewayRequest;

#[derive(Clone)]
struct GatewayState {
    limiter: Arc<RateLimiter>,
    pipeline: Arc<middleware::Pipeline>,
    proxies: HashMap<String, Proxy>,
}

/// Run gateway with default configuration (uses env vars for route modes)
pub async fn run(config: &GatewayConfig) -> anyhow::Result<()> {
    run_with_routers(config, HashMap::new()).await
}

/// Run gateway with embedded routers for specific routes
/// Routes not in `routers` map will be proxied based on config
pub async fn run_with_routers(
    config: &GatewayConfig,
    routers: HashMap<String, Router>,
) -> anyhow::Result<()> {
    println!("gateway listening on {}", config.listen_addr);

    let limiter = Arc::new(RateLimiter::new(100));
    let pipeline = Arc::new(middleware::default_pipeline());

    // Build proxies for routes that are in proxy mode and not embedded
    let mut proxies = HashMap::new();
    for (route, route_config) in &config.routes {
        let should_proxy = route_config.mode == RouteMode::Proxy || !routers.contains_key(route);
        if should_proxy && !route_config.upstream_base.is_empty() {
            println!(
                "  route {} -> proxy to {}",
                route, route_config.upstream_base
            );
            proxies.insert(route.clone(), Proxy::new(&route_config.upstream_base));
        } else if routers.contains_key(route) {
            println!("  route {} -> embedded", route);
        }
    }

    let state = Arc::new(GatewayState {
        limiter,
        pipeline,
        proxies,
    });

    // Build the router
    let mut app = Router::new();

    // Add embedded routers
    for (route, router) in routers {
        app = app.nest(&route, router);
    }

    // Add proxy routes for remaining routes
    for (route, _) in &state.proxies {
        let route_path = route.clone();
        let route_any = route.clone();
        let route_wildcard = format!("{}{{*path}}", route);

        app = app
            .route(
                &route_any,
                any(move |ext, req| proxy_route(ext, req, route_path.clone())),
            )
            .route(
                &route_wildcard,
                any({
                    let route = route.clone();
                    move |ext, req| proxy_route(ext, req, route.clone())
                }),
            );
    }

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
    let start = Instant::now();
    let path = req.uri().path().to_string();
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
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|v| (name.to_string(), v.to_string()))
        })
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

    let response = next.run(req).await;
    let status = response.status();
    let elapsed_ms = start.elapsed().as_millis();
    println!("[gateway] {} {} {}ms", status.as_u16(), path, elapsed_ms);
    Ok(response)
}

async fn proxy_route(
    Extension(state): Extension<Arc<GatewayState>>,
    req: Request<Body>,
    route: String,
) -> Response<Body> {
    let Some(proxy) = state.proxies.get(&route) else {
        return (StatusCode::BAD_GATEWAY, format!("proxy not configured for {}", route))
            .into_response();
    };

    match proxy.forward(req, &route).await {
        Ok(response) => response,
        Err(err) => bad_gateway(format!("upstream error: {err}")),
    }
}
