fn main() {
    common::init_service("gateway");
    gateway_core::run();
}
