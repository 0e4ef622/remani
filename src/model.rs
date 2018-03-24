//! A module that handles window update events

use piston::input::{ UpdateArgs, Button };

use config::Config;

/// Holds game states needed by the logic and renderer
pub struct Model {
    pub keys_down: [bool; 7],
}

impl Model {

    /// Create a model for the game controller
    pub fn new() -> Self {
        Self {
            keys_down: [false; 7]
        }
    }

    /// Called when an update event occurs
    pub fn update(&mut self, args: &UpdateArgs) {
        // stuff
    }

    /// Called when a press event occurs
    pub fn press(&mut self, args: &Button, config: &Config) {
        config.key_bindings.iter().zip(self.keys_down.iter_mut())
            .for_each(|(key_binding, key_down)| {
                if *args == *key_binding { *key_down = true; }
            });
    }

    pub fn release(&mut self, args: &Button, config: &Config) {
        config.key_bindings.iter().zip(self.keys_down.iter_mut())
            .for_each(|(key_binding, key_down)| {
                if *args == *key_binding { *key_down = false; }
            });
    }

}
