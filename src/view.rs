//! A module that handles window render events

use piston::input::RenderArgs;
use opengl_graphics::GlGraphics;
use graphics;
use graphics::draw_state::DrawState;
use skin::Skin;
use model::Model;

use chart;
use config::Config;

/// Holds values and resources needed by the window to do drawing stuff
pub struct View {
    pub gl: GlGraphics,
    skin: Box<Skin>,
    draw_state: DrawState,
    chart: chart::Chart,
    next_note_index: usize,
    notes_on_screen_indices: Vec<usize>,
    /// Indices of the notes in notes_on_screen that are actually below the screen and need to be
    /// removed
    notes_below_screen_indices: Vec<usize>,
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
            next_note_index: 0,
            notes_on_screen_indices: Vec::with_capacity(128),
            notes_below_screen_indices: Vec::with_capacity(128),
        }
    }

    /// Called when a render event occurs
    pub fn render(&mut self, args: &RenderArgs, config: &Config, model: &Model, time: f64) {
        let skin = &self.skin;
        let draw_state = &self.draw_state;
        let chart = &self.chart;
        let next_note_index = &mut self.next_note_index;
        let notes_on_screen_indices = &mut self.notes_on_screen_indices;
        let notes_below_screen_indices = &mut self.notes_below_screen_indices;

        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear([0.0; 4], gl);

            skin.draw_track(draw_state, c.transform, gl, args.height as f64);

            let mut add_next_note_index = 0;

            for (index, note) in chart.notes[*next_note_index..].iter().enumerate() {
                if note.time - time > 1.0 / config.scroll_speed { break; }

                notes_on_screen_indices.push(index + *next_note_index);
                add_next_note_index += 1;
            }
            *next_note_index += add_next_note_index;

            for (index, &note_index) in notes_on_screen_indices.iter().enumerate() {

                let note = &chart.notes[note_index];
                if let Some(end_time) = note.end_time {
                    if end_time - time < 0.0 {
                        notes_below_screen_indices.push(index);
                        continue;
                    }
                    skin.draw_long_note(draw_state, c.transform, gl, args.height as f64, (note.time - time) * config.scroll_speed, (end_time - time) * config.scroll_speed, note.column);
                } else {
                    if note.time - time < 0.0 {
                        notes_below_screen_indices.push(index);
                        continue;
                    }
                    skin.draw_note(draw_state, c.transform, gl, args.height as f64, (note.time - time) * config.scroll_speed, note.column);
                }
            }

            for &index in notes_below_screen_indices.iter().rev() {
                notes_on_screen_indices.swap_remove(index);
            }
            notes_below_screen_indices.clear();

            skin.draw_keys(draw_state, c.transform, gl, args.height as f64, &model.keys_down);
        });

    }

}
