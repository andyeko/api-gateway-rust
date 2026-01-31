pub mod config;
pub mod service;

pub fn run() {
    let config = config::AuthConfig::default();
    service::run(&config);
}
