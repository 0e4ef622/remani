//! Handles configuration of the game

use piston::input;
use std::path;

/// Holds all the configuration values like skin path or key bindings
pub struct Config {
    pub key_bindings: [input::Button; 7],
    pub default_osu_skin_path: path::PathBuf,
    pub skin_path: path::PathBuf,
    pub scroll_speed: u32,
}

/// Load configuration from a file
pub fn get_config() -> Config {
    use piston::input::keyboard::Key;
    use piston::input::Button::Keyboard;

    // TODO

    Config {
        key_bindings: [
            Keyboard(Key::S),
            Keyboard(Key::D),
            Keyboard(Key::F),
            Keyboard(Key::Space),
            Keyboard(Key::J),
            Keyboard(Key::K),
            Keyboard(Key::L),
        ],
        default_osu_skin_path: path::PathBuf::from("default_osu_skin"),
        skin_path: path::PathBuf::from("test/test_skin"),
        scroll_speed: 1,
    }
}
