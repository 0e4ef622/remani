//! A module that handles window update events

use piston::input::{ UpdateArgs, Button };

/// Holds game states needed by the logic and renderer
pub struct Model {
    pub key1_down: bool,
    key2_down: bool,
    key3_down: bool,
    key4_down: bool,
    key5_down: bool,
    key6_down: bool,
    key7_down: bool,
}

impl Model {

    /// Create a model for the game controller
    pub fn new() -> Self {
        Self {
            key1_down: false,
            key2_down: false,
            key3_down: false,
            key4_down: false,
            key5_down: false,
            key6_down: false,
            key7_down: false,
        }
    }

    /// Called when an update event occurs
    pub fn update(&mut self, args: &UpdateArgs) {
        // stuff
    }

    /// Called when a press event occurs
    pub fn press(&mut self, args: &Button) {
        match *args {
            Button::Keyboard(k) => println!("Keyboard event {:?}", k),
            Button::Mouse(k) => println!("Mouse event {:?}", k),
            _ => panic!("uhhhh"),
        }
    }

}
