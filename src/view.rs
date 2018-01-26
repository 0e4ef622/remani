extern crate graphics;
extern crate opengl_graphics;
extern crate texture;

use piston::input::{ RenderArgs, UpdateArgs, Button };
use opengl_graphics::{ GlGraphics, Texture };
use self::texture::TextureSettings;
use graphics::image::Image;
use graphics::draw_state::DrawState;
use std::path::Path;

pub struct View {
    pub gl: GlGraphics,
    texture: Texture,
    image: Image,
    draw_state: DrawState,
}

impl View {

    pub fn new(gl: GlGraphics, path: &Path) -> Self {
        View {
            gl: gl,
            texture: Texture::from_path(path, &TextureSettings::new()).expect("Failed to load image"),
            image: Image::new().rect(graphics::rectangle::square(50.0, 50.0, 100.0)),
            draw_state: DrawState { scissor: None, stencil: None, blend: None },
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        let image = &self.image;
        let texture = &self.texture;
        let draw_state = &self.draw_state;
        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear([1.0; 4], gl);
            image.draw(texture, draw_state, c.transform, gl);
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
