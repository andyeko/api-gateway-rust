#[derive(Debug, Clone)]
pub struct AdminConfig {
    pub bind_addr: String,
    pub database_url: String,
}

impl Default for AdminConfig {
    fn default() -> Self {
        let bind_addr =
            std::env::var("ADMIN_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:4001".to_string());
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/apisentinel".to_string()
        });

        Self {
            bind_addr,
            database_url,
        }
    }
}
