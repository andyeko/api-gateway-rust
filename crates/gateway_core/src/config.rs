use std::collections::HashMap;

/// Mode for handling a route - either embed the handler or proxy to upstream
#[derive(Debug, Clone, PartialEq)]
pub enum RouteMode {
    /// Handle requests in-process with an embedded router
    Embedded,
    /// Proxy requests to an upstream service
    Proxy,
}

impl RouteMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "proxy" => RouteMode::Proxy,
            _ => RouteMode::Embedded,
        }
    }
}

/// Configuration for a single route
#[derive(Debug, Clone)]
pub struct RouteConfig {
    pub mode: RouteMode,
    pub upstream_base: String,
}

impl RouteConfig {
    pub fn embedded() -> Self {
        Self {
            mode: RouteMode::Embedded,
            upstream_base: String::new(),
        }
    }

    pub fn proxy(upstream_base: impl Into<String>) -> Self {
        Self {
            mode: RouteMode::Proxy,
            upstream_base: upstream_base.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub listen_addr: String,
    /// Route configurations keyed by base path (e.g., "/admin", "/auth")
    pub routes: HashMap<String, RouteConfig>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        let mut routes = HashMap::new();

        // Admin route configuration
        let admin_mode = std::env::var("GATEWAY_ADMIN_MODE")
            .map(|s| RouteMode::from_str(&s))
            .unwrap_or(RouteMode::Embedded);
        let admin_upstream = std::env::var("GATEWAY_ADMIN_UPSTREAM")
            .unwrap_or_else(|_| "http://localhost:4001".to_string());
        routes.insert(
            "/admin".to_string(),
            RouteConfig {
                mode: admin_mode,
                upstream_base: admin_upstream,
            },
        );

        // Auth route configuration
        let auth_mode = std::env::var("GATEWAY_AUTH_MODE")
            .map(|s| RouteMode::from_str(&s))
            .unwrap_or(RouteMode::Embedded);
        let auth_upstream = std::env::var("GATEWAY_AUTH_UPSTREAM")
            .unwrap_or_else(|_| "http://localhost:4002".to_string());
        routes.insert(
            "/auth".to_string(),
            RouteConfig {
                mode: auth_mode,
                upstream_base: auth_upstream,
            },
        );

        Self {
            listen_addr: std::env::var("GATEWAY_LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            routes,
        }
    }
}

impl GatewayConfig {
    /// Check if a route should be proxied
    pub fn is_proxy(&self, route: &str) -> bool {
        self.routes
            .get(route)
            .map(|r| r.mode == RouteMode::Proxy)
            .unwrap_or(false)
    }

    /// Get upstream URL for a route (if in proxy mode)
    pub fn get_upstream(&self, route: &str) -> Option<&str> {
        self.routes.get(route).and_then(|r| {
            if r.mode == RouteMode::Proxy {
                Some(r.upstream_base.as_str())
            } else {
                None
            }
        })
    }

    /// Set a route to embedded mode with a router
    pub fn set_embedded(&mut self, route: &str) {
        if let Some(r) = self.routes.get_mut(route) {
            r.mode = RouteMode::Embedded;
        }
    }

    /// Set a route to proxy mode with upstream URL
    pub fn set_proxy(&mut self, route: &str, upstream: impl Into<String>) {
        self.routes.insert(
            route.to_string(),
            RouteConfig::proxy(upstream),
        );
    }
}
