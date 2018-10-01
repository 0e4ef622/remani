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

fn main() {
    println!("{:?}", ProjectDirs::from("", "0e4ef622", "Remani").unwrap().config_dir());
    let config = config::get_config();
    window::start(config);
}
