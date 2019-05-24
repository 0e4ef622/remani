#![deny(rust_2018_idioms)]
#![feature(inner_deref)]

macro_rules! remani_warn {
    ($fmt:expr) => (eprintln!(concat!("WARNING: ", $fmt)));
    ($fmt:expr, $($arg:tt)*) => {
        eprintln!(concat!("WARNING: ", $fmt), $($arg)*)
    };
}

pub mod audio;
pub mod chart;
pub mod config;
pub mod judgement;
pub mod gameskin;
pub mod window;
