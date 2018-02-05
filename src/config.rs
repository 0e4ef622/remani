//! Handles configuration of the game
extern crate piston;

use piston::input;
use std::path;

pub struct KeyBindings {
    pub key1: input::Button,
    pub key2: input::Button,
    pub key3: input::Button,
    pub key4: input::Button,
    pub key5: input::Button,
    pub key6: input::Button,
    pub key7: input::Button,
}

/// Holds all the configuration values like skin path or key bindings
pub struct Config {
    pub key_bindings: KeyBindings,
    pub skin_path: path::PathBuf,
    pub scroll_speed: u32,
}

/// Load configuration from a file
pub fn get_config() -> Config {
    use piston::input::keyboard::Key;
    use piston::input::Button::Keyboard;

    // TODO

    Config {
        key_bindings: KeyBindings {
            key1: Keyboard(Key::A),
            key2: Keyboard(Key::S),
            key3: Keyboard(Key::D),
            key4: Keyboard(Key::Space),
            key5: Keyboard(Key::J),
            key6: Keyboard(Key::K),
            key7: Keyboard(Key::L),
        },
        skin_path: path::PathBuf::from("test_skin"),
        scroll_speed: 1,
    }
}
