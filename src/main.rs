use remani::{config, window};

fn main() {
    let config_path = config::config_path();
    let config = config::get_config(&config_path);
    window::start(config);
}
