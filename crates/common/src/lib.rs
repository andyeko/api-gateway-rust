pub fn init_service(name: &str) {
    println!("starting service: {name}");
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
}
