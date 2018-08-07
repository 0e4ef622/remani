#![feature(macro_literal_matcher)]

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
mod skin;
mod window;

fn main() {
    let config = config::get_config();
    window::start(config);
}
