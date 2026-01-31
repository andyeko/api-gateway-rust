#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub listen_addr: String,
    pub upstream_base: String,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:8080".to_string(),
            upstream_base: "http://localhost:9000".to_string(),
        }
    }
}
