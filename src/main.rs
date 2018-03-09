extern crate piston;
extern crate texture;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate image;
extern crate cpal;

#[cfg(feature="mp3")]
extern crate simplemad;

mod model;
mod view;
mod chart;
mod skin;
mod config;
mod game;
mod audio;

fn main() {
    let config = config::get_config();
    game::start(config);
}
