#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub listen_addr: String,
    pub admin_upstream_base: String,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:8080".to_string(),
            admin_upstream_base: std::env::var("ADMIN_UPSTREAM_BASE")
                .unwrap_or_else(|_| "http://localhost:4001".to_string()),
        }
    }
}
