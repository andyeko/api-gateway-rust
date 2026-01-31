#[tokio::main]
async fn main() {
    common::init_service("gateway");
    if let Err(err) = gateway_core::run().await {
        eprintln!("gateway service error: {err}");
    }
}
