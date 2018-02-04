//! A module that handles window events

extern crate graphics;
extern crate opengl_graphics;
extern crate texture;

use piston::input::{ RenderArgs, UpdateArgs, Button };
use opengl_graphics::GlGraphics;
use graphics::image::Image;
use graphics::draw_state::DrawState;
use skin::Skin;
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
    pub fn render(&mut self, args: &RenderArgs) {
        let image = &self.image;
        let skin = &self.skin;
        let draw_state = &self.draw_state;
        println!("draw size: {:?}, window size: {:?}", args.viewport().draw_size, args.viewport().window_size);
        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear([1.0; 4], gl);
            image.draw(&skin.miss[0], draw_state, c.transform, gl);
            image.draw(&skin.hit50[0], draw_state, c.transform.trans(100., 0.), gl);
            image.draw(&skin.hit100[0], draw_state, c.transform.trans(200., 0.), gl);
            image.draw(&skin.hit300[0], draw_state, c.transform.trans(300., 0.), gl);
            image.draw(&skin.hit300g[0], draw_state, c.transform.trans(400., 0.), gl);

            image.draw(skin.keys[0][0].deref(), draw_state, c.transform.trans(0., 100.), gl);
            image.draw(skin.keys[1][0].deref(), draw_state, c.transform.trans(100., 100.), gl);
            image.draw(skin.keys[3][0].deref(), draw_state, c.transform.trans(200., 100.), gl);

            image.draw(skin.keys_d[0][0].deref(), draw_state, c.transform.trans(0., 200.), gl);
            image.draw(skin.keys_d[1][0].deref(), draw_state, c.transform.trans(100., 200.), gl);
            image.draw(skin.keys_d[3][0].deref(), draw_state, c.transform.trans(200., 200.), gl);

            image.draw(skin.notes[0][0].deref(), draw_state, c.transform.trans(0., 300.), gl);
            image.draw(skin.notes[1][0].deref(), draw_state, c.transform.trans(100., 300.), gl);
            image.draw(skin.notes[3][0].deref(), draw_state, c.transform.trans(200., 300.), gl);

            image.draw(skin.long_notes_head[0][0].deref(), draw_state, c.transform.trans(0., 400.), gl);
            image.draw(skin.long_notes_head[1][0].deref(), draw_state, c.transform.trans(100., 400.), gl);
            image.draw(skin.long_notes_head[3][0].deref(), draw_state, c.transform.trans(200., 400.), gl);

            image.draw(skin.long_notes_body[0][0].deref(), draw_state, c.transform.trans(0., 500.), gl);
            image.draw(skin.long_notes_body[1][0].deref(), draw_state, c.transform.trans(100., 500.), gl);
            image.draw(skin.long_notes_body[3][0].deref(), draw_state, c.transform.trans(200., 500.), gl);
        });
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
