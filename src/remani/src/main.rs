use directories::ProjectDirs;
use std::env;
use std::path::PathBuf;

use remani_judgement as judgement;
use remani_gameskin as gameskin;
use remani_chart as chart;
use remani_config as config;
use remani_audio as audio;
use remani_warn::remani_warn;

mod window;

fn main() {

    let config_path: PathBuf = env::var_os("REMANI_CONF")
        .map(|s| s.into())
        .unwrap_or(ProjectDirs::from("", "0e4ef622", "Remani").unwrap().config_dir().join("config.toml"));

    let config = config::get_config(&config_path);
    window::start(config);
}
