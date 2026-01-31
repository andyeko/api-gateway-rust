/// Lightweight observability hooks shared across services.
/// Replace with tracing/logging crates when ready.
pub fn init_observability(service_name: &str) {
    println!("observability initialized for: {service_name}");
}

pub fn event(message: &str) {
    println!("event: {message}");
}
