#[tokio::main]
async fn main() {
    common::init_service("auth");
    if let Err(err) = auth_core::run().await {
        eprintln!("auth service error: {err}");
    }
}
