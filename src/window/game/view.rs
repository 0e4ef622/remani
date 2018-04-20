//! A module that handles window render events

use piston::input::RenderArgs;
use opengl_graphics::GlGraphics;
use graphics::draw_state::DrawState;
use graphics;

use skin::Skin;
use super::Model;
use config::Config;
use judgement::Judgement;

use chart;

/// Holds values and resources needed by the window to do drawing stuff
pub struct View {
    skin: Box<Skin>,
    draw_state: DrawState,
    next_note_index: usize,
    notes_on_screen_indices: Vec<usize>,
    /// Indices of the notes in notes_on_screen that are actually below the screen and need to be
    /// removed
    notes_below_screen_indices: Vec<usize>,
    notes_pos: Vec<(usize, f64, Option<f64>)>,
}

impl View {

    /// Create a view with some hardcoded defaults and stuffs
    pub fn new(skin: Box<Skin>) -> View {
        let draw_state = DrawState::default();
        View {
            skin,
            draw_state,
            next_note_index: 0,
            notes_on_screen_indices: Vec::with_capacity(128),
            notes_below_screen_indices: Vec::with_capacity(128),
            notes_pos: Vec::with_capacity(128),
        }
    }

    /// Called when a render event occurs
    pub fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, config: &Config, chart: &chart::Chart, model: &Model, time: f64) {
        let skin = &mut self.skin;
        let draw_state = &self.draw_state;
        let next_note_index = &mut self.next_note_index;
        let notes_on_screen_indices = &mut self.notes_on_screen_indices;
        let notes_below_screen_indices = &mut self.notes_below_screen_indices;
        let notes_pos = &mut self.notes_pos;

        gl.draw(args.viewport(), |c, gl| {
            graphics::clear([0.0; 4], gl);

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
                } else {
                    if note.time - time < 0.0 {
                        notes_below_screen_indices.push(index);
                        continue;
                    }
                }
            }

            for &index in notes_below_screen_indices.iter().rev() {
                notes_on_screen_indices.swap_remove(index);
            }
            notes_below_screen_indices.clear();
            notes_pos.clear();
            notes_pos.extend(notes_on_screen_indices.iter().map(|&i| {
                let note = &chart.notes[i];
                let pos = (note.time - time) * config.scroll_speed;
                let end_pos = note.end_time.map(|t| (t - time) * config.scroll_speed);
                (note.column, pos, end_pos)
            }));

            skin.draw_play_scene(draw_state,
                                 c.transform,
                                 gl,
                                 args.height as f64,
                                 &model.keys_down,
                                 &notes_pos[..]);
        });

    }

    pub fn draw_judgement(&mut self, column: usize, judgement: Judgement) {
        self.skin.draw_judgement(column, judgement);
    }

}
