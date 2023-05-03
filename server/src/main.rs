fn main() {
    shared::init_env();
    if let Err(err) = server::run() {
        shared::log::error!("Server terminated: ${err}");
    }
}
