//! A module that handles window render events

use piston::input::RenderArgs;
use opengl_graphics::GlGraphics;
use graphics;
use graphics::draw_state::DrawState;
use skin::Skin;
use model::Model;

use chart;

/// Holds values and resources needed by the window to do drawing stuff
pub struct View {
    pub gl: GlGraphics,
    skin: Box<Skin>,
    draw_state: DrawState,
    chart: chart::Chart,
    note_index: usize,
}

impl View {

    /// Create a view with some hardcoded defaults and stuffs
    pub fn new(gl: GlGraphics, skin: Box<Skin>, chart: chart::Chart) -> Self {
        let gl = gl;
        let draw_state = DrawState::default();

        Self {
            gl,
            skin,
            draw_state,
            chart,
            note_index: 0,
        }
    }

    /// Called when a render event occurs
    pub fn render(&mut self, args: &RenderArgs, model: &Model, time: f64) {
        let skin = &self.skin;
        let draw_state = &self.draw_state;
        let chart = &self.chart;
        let note_index = &mut self.note_index;

        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear([0.0; 4], gl);

            skin.draw_track(draw_state, c.transform, gl, args.height as f64);

            for note in &chart.notes[*note_index..] {
                if note.time - time > 1.0 { break; }
                if note.time - time < -1.0 { *note_index += 1; continue; }
                skin.draw_note(draw_state, c.transform, gl, args.height as f64, (note.time - time) * 1.3, note.column);
            }

            skin.draw_keys(draw_state, c.transform, gl, args.height as f64, &model.keys_down);
        });

    }

}
