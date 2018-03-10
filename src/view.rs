//! A module that handles window render events

use piston::input::RenderArgs;
use opengl_graphics::GlGraphics;
use graphics;
use graphics::draw_state::DrawState;
use skin::Skin;
use model::Model;

/// Holds values and resources needed by the window to do drawing stuff
pub struct View {
    pub gl: GlGraphics,
    skin: Box<Skin>,
    draw_state: DrawState,
}

impl View {

    /// Create a view with some hardcoded defaults and stuffs
    pub fn new(gl: GlGraphics, skin: Box<Skin>) -> Self {
        let gl = gl;
        let draw_state = DrawState::default();

        Self {
            gl: gl,
            skin: skin,
            draw_state: draw_state,
        }
    }

    /// Called when a render event occurs
    pub fn render(&mut self, args: &RenderArgs, model: &Model) {
        let skin = &self.skin;
        let draw_state = &self.draw_state;
        //println!("draw size: {:?}, window size: {:?}", args.viewport().draw_size, args.viewport().window_size);
        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear([0.0; 4], gl);

            skin.draw_track(draw_state, c.transform, gl);
            skin.draw_note(draw_state, c.transform, gl, 5.0, 2);

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
            if model.keys_down[0] { red_rect.draw(key1, draw_state, c.transform, gl); }
            if model.keys_down[1] { red_rect.draw(key2, draw_state, c.transform, gl); }
            if model.keys_down[2] { red_rect.draw(key3, draw_state, c.transform, gl); }
            if model.keys_down[3] { red_rect.draw(key4, draw_state, c.transform, gl); }
            if model.keys_down[4] { red_rect.draw(key5, draw_state, c.transform, gl); }
            if model.keys_down[5] { red_rect.draw(key6, draw_state, c.transform, gl); }
            if model.keys_down[6] { red_rect.draw(key7, draw_state, c.transform, gl); }
        });

    }

}
