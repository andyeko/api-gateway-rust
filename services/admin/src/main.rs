#[tokio::main]
async fn main() {
    common::init_service("admin");
    if let Err(err) = admin_core::run().await {
        eprintln!("admin service error: {err}");
    }
}
