extern crate piston;
extern crate texture;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate image;

mod model;
mod view;
mod chart;
mod skin;
mod config;
mod game;

fn main() {
    let config = config::get_config();
    game::start(config);
}
