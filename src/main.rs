extern crate piston;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;

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
