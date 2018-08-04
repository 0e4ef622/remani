#![feature(macro_literal_matcher)]

macro_rules! remani_warn {
    ($fmt:expr) => (eprintln!(concat!("WARNING: ", $fmt)));
    ($fmt:expr, $($arg:tt)*) => {
        eprintln!(concat!("WARNING: ", $fmt), $($arg)*)
    };
}


mod chart;
mod skin;
mod config;
mod audio;
mod window;
mod judgement;

fn main() {
    let config = config::get_config();
    window::start(config);
}
