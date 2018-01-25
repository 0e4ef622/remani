extern crate graphics;
extern crate opengl_graphics;

use piston::input::{ RenderArgs, UpdateArgs, Button };

pub struct View {
    pub gl: opengl_graphics::GlGraphics,
}

impl View {
    pub fn render(&mut self, args: &RenderArgs) {
        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear([1.0; 4], gl);

        });
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        // stuff
    }

    pub fn press(&mut self, args: &Button) {
        match args {
            &Button::Keyboard(k) => println!("Keyboard event {:?}", k),
            &Button::Mouse(k) => println!("Mouse event {:?}", k),
            _ => panic!("uhhhh"),
        }
    }

}
