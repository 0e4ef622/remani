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
            skin.draw_keys(draw_state, c.transform, gl, &model.keys_down);
        });

    }

}
