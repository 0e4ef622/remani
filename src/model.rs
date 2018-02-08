//! A module that handles window update events

use piston::input::{ UpdateArgs, Button };

use config::Config;

/// Holds game states needed by the logic and renderer
pub struct Model {
    pub key1_down: bool,
    pub key2_down: bool,
    pub key3_down: bool,
    pub key4_down: bool,
    pub key5_down: bool,
    pub key6_down: bool,
    pub key7_down: bool,
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
    pub fn press(&mut self, args: &Button, config: &Config) {
        if *args == config.key_bindings.key1 { self.key1_down = true; }
        if *args == config.key_bindings.key2 { self.key2_down = true; }
        if *args == config.key_bindings.key3 { self.key3_down = true; }
        if *args == config.key_bindings.key4 { self.key4_down = true; }
        if *args == config.key_bindings.key5 { self.key5_down = true; }
        if *args == config.key_bindings.key6 { self.key6_down = true; }
        if *args == config.key_bindings.key7 { self.key7_down = true; }
        /*match *args {
            Button::Keyboard(k) => println!("Keyboard press event {:?}", k),
            Button::Mouse(k) => println!("Mouse press event {:?}", k),
            _ => (),
        }*/
    }

    pub fn release(&mut self, args: &Button, config: &Config) {
        if *args == config.key_bindings.key1 { self.key1_down = false; }
        if *args == config.key_bindings.key2 { self.key2_down = false; }
        if *args == config.key_bindings.key3 { self.key3_down = false; }
        if *args == config.key_bindings.key4 { self.key4_down = false; }
        if *args == config.key_bindings.key5 { self.key5_down = false; }
        if *args == config.key_bindings.key6 { self.key6_down = false; }
        if *args == config.key_bindings.key7 { self.key7_down = false; }
        /*match *args {
            Button::Keyboard(k) => println!("Keyboard release event {:?}", k),
            Button::Mouse(k) => println!("Mouse release event {:?}", k),
            _ => (),
        }*/
    }

}
