#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub listen_addr: String,
    pub jwt_secret: String,
    pub issuer: String,
    /// Token validity duration in seconds (default: 300 = 5 minutes)
    pub token_ttl_seconds: u64,
    /// Default admin email (only works when no users exist)
    pub default_admin_email: Option<String>,
    /// Default admin password (only works when no users exist)
    pub default_admin_password: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            listen_addr: std::env::var("AUTH_LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:4002".to_string()),
            jwt_secret: std::env::var("AUTH_JWT_SECRET")
                .unwrap_or_else(|_| "change-me-in-production-super-secret-key".to_string()),
            issuer: std::env::var("AUTH_ISSUER")
                .unwrap_or_else(|_| "apisentinel".to_string()),
            token_ttl_seconds: std::env::var("AUTH_TOKEN_TTL_SECONDS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300), // 5 minutes
            default_admin_email: std::env::var("AUTH_DEFAULT_ADMIN_EMAIL").ok(),
            default_admin_password: std::env::var("AUTH_DEFAULT_ADMIN_PASSWORD").ok(),
        }
    }
}
