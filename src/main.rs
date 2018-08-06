#![deny(bare_trait_objects)]
#![feature(macro_literal_matcher)]

extern crate piston;
extern crate texture;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate image;
extern crate cpal;

#[cfg(feature="mp3")]
extern crate simplemad;

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
