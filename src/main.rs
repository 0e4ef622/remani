extern crate piston;
extern crate texture;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate image;
extern crate cpal;

#[cfg(feature="mp3")]
extern crate simplemad;

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
