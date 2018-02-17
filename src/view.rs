//! A module that handles window render events

extern crate graphics;
extern crate opengl_graphics;
extern crate texture;

use piston::input::{ RenderArgs, Button };
use opengl_graphics::GlGraphics;
use graphics::image::Image;
use graphics::draw_state::DrawState;
use skin::Skin;
use model::Model;
use graphics::Transformed;
use std::ops::Deref;

/// Holds values and resources needed by the window to do drawing stuff
pub struct View {
    pub gl: GlGraphics,
    skin: Skin,
    image: Image,
    draw_state: DrawState,
}

impl View {

    /// Create a view with some hardcoded defaults and stuffs
    pub fn new(gl: GlGraphics, skin: Skin) -> Self {
        let gl = gl;
        let image = Image::new().rect(graphics::rectangle::square(50.0, 50.0, 100.0));
        let draw_state = DrawState::default();

        Self {
            gl: gl,
            skin: skin,
            image: image,
            draw_state: draw_state,
        }
    }

    /// Called when a render event occurs
    pub fn render(&mut self, args: &RenderArgs, model: &Model) {
        let image = &self.image;
        let skin = &self.skin;
        let draw_state = &self.draw_state;
        //println!("draw size: {:?}, window size: {:?}", args.viewport().draw_size, args.viewport().window_size);
        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear([1.0; 4], gl);

            skin.draw_stage(draw_state, &c.transform, gl);

            let red_rect = graphics::rectangle::Rectangle::new([1.0, 0.0, 0.0, 1.0]);
            let bordor = graphics::rectangle::Rectangle::new_border([1.0, 0.0, 0.0, 1.0], 2.0).color([1.0; 4]);
            let key1 = [100.0, args.height as f64 - 100.0, 60.0, 30.0];
            let key2 = [170.0, args.height as f64 - 100.0, 60.0, 30.0];
            let key3 = [240.0, args.height as f64 - 100.0, 60.0, 30.0];
            let key4 = [310.0, args.height as f64 - 100.0, 60.0, 30.0];
            let key5 = [380.0, args.height as f64 - 100.0, 60.0, 30.0];
            let key6 = [450.0, args.height as f64 - 100.0, 60.0, 30.0];
            let key7 = [520.0, args.height as f64 - 100.0, 60.0, 30.0];
            bordor.draw(key1, draw_state, c.transform, gl);
            bordor.draw(key2, draw_state, c.transform, gl);
            bordor.draw(key3, draw_state, c.transform, gl);
            bordor.draw(key4, draw_state, c.transform, gl);
            bordor.draw(key5, draw_state, c.transform, gl);
            bordor.draw(key6, draw_state, c.transform, gl);
            bordor.draw(key7, draw_state, c.transform, gl);
            if model.key1_down { red_rect.draw(key1, draw_state, c.transform, gl); }
            if model.key2_down { red_rect.draw(key2, draw_state, c.transform, gl); }
            if model.key3_down { red_rect.draw(key3, draw_state, c.transform, gl); }
            if model.key4_down { red_rect.draw(key4, draw_state, c.transform, gl); }
            if model.key5_down { red_rect.draw(key5, draw_state, c.transform, gl); }
            if model.key6_down { red_rect.draw(key6, draw_state, c.transform, gl); }
            if model.key7_down { red_rect.draw(key7, draw_state, c.transform, gl); }
        });

    }

}
