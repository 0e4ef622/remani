use directories::ProjectDirs;
use std::env;
use std::path::PathBuf;

use remani::{config, window};

fn main() {

    let config_path: PathBuf = env::var_os("REMANI_CONF")
        .map(|s| s.into())
        .unwrap_or(ProjectDirs::from("", "0e4ef622", "Remani").unwrap().config_dir().join("config.toml"));

    let config = config::get_config(&config_path);
    window::start(config);
}
