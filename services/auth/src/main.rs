fn main() {
    common::init_service("auth");
    auth_core::run();
}
