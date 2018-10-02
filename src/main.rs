#![feature(macro_literal_matcher)]
#![feature(custom_attribute)]

macro_rules! remani_warn {
    ($fmt:expr) => (eprintln!(concat!("WARNING: ", $fmt)));
    ($fmt:expr, $($arg:tt)*) => {
        eprintln!(concat!("WARNING: ", $fmt), $($arg)*)
    };
}

mod audio;
mod chart;
mod config;
mod judgement;
mod gameskin;
mod window;

use directories::ProjectDirs;
use std::env;
use std::path::PathBuf;

fn main() {

    let config_path: PathBuf = env::var_os("REMANI_CONF")
        .map(|s| s.into())
        .unwrap_or(ProjectDirs::from("", "0e4ef622", "Remani").unwrap().config_dir().join("remani.conf"));

    let config = config::get_config(&config_path);
    window::start(config);
}
