extern crate graphics;
extern crate opengl_graphics;

use piston::input::RenderArgs;

pub struct RemaniWindow {
    pub gl: opengl_graphics::GlGraphics,
}

impl RemaniWindow {
    pub fn render(&mut self, args: &RenderArgs) {
        //use graphics::*;
        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear([1.0; 4], gl);
        });
    }
}
