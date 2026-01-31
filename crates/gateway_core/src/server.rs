use crate::config::GatewayConfig;
use crate::middleware;
use crate::proxy::Proxy;
use crate::rate_limit::RateLimiter;
use crate::types::Request;

pub async fn run(config: &GatewayConfig) -> anyhow::Result<()> {
    println!("gateway listening on {}", config.listen_addr);
    println!("proxying to {}", config.upstream_base);

    let limiter = RateLimiter::new(100);
    let pipeline = middleware::default_pipeline();
    let proxy = Proxy::new(config.upstream_base.clone());

    let mut request = Request::new("/");
    request
        .headers
        .push(("x-api-key".to_string(), "dev".to_string()));

    if !limiter.allow("127.0.0.1") {
        println!("rate limited request");
        return Ok(());
    }

    match middleware::apply(&pipeline, request) {
        Ok(req) => {
            let response = proxy.forward(req);
            println!("response: {}", response.body);
        }
        Err(res) => {
            println!("blocked: {}", res.body);
        }
    }
    
    Ok(())
}
