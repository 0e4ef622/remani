//! A module that handles window render events for the game scene

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

    /// Index of the next note that isn't on the screen yet
    next_note_index: usize,
    current_timing_point_index: usize,

    notes_on_screen_indices: Vec<usize>,
    /// Indices of the notes in notes_on_screen that are actually below the screen and need to be
    /// removed
    notes_below_screen_indices: Vec<usize>,

    /// (index, start_pos, end_pos)
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
            current_timing_point_index: 0,
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
        let current_timing_point_index = &mut self.current_timing_point_index;

        gl.draw(args.viewport(), |c, gl| {
            graphics::clear([0.0; 4], gl);

            let mut add_next_note_index = 0;

            for (index, note) in chart.notes[*next_note_index..].iter().enumerate() {

                let note_pos = calc_pos(time, note.time, chart, config.scroll_speed, *current_timing_point_index);
                if note_pos > 1.0 { break; }

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

            // TODO manage self.current_timing_point_index

            for &index in notes_below_screen_indices.iter().rev() {
                notes_on_screen_indices.swap_remove(index);
            }
            notes_below_screen_indices.clear();
            notes_pos.clear();
            notes_pos.extend(notes_on_screen_indices.iter().map(|&i| {
                let note = &chart.notes[i];

                let pos = calc_pos(time, note.time, chart, config.scroll_speed, *current_timing_point_index);
                let end_pos = note.end_time.map(|t| calc_pos(time, t, chart, config.scroll_speed, *current_timing_point_index));

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

    pub fn key_down(&mut self, column: usize) {
        self.skin.key_down(column);
    }

    pub fn key_up(&mut self, column: usize) {
        self.skin.key_up(column);
    }
}

/// Given the time in seconds from the start of the song, calculate the position, taking into
/// account SV changes. Return value is an f64 between 0.0 and 1.0, 0.0 being at the judgement
/// line, and 1.0 being at the top of the stage.
///
/// Used to calculate note position.
fn calc_pos(current_time: f64, time: f64, chart: &chart::Chart, scroll_speed: f64, current_timing_point_index: usize) -> f64 {
    let mut iterator = chart.timing_points[current_timing_point_index..].iter()
        .take_while(|tp| tp.offset < time)
        .filter(|tp| tp.is_sv())
        .peekable();

    // TODO BPM changes also affect SV

    let mut last_tp = None;
    while iterator.peek().is_some() {
        if iterator.peek().unwrap().offset < current_time {
            last_tp = Some(iterator.next().unwrap());
        } else {
            break;
        }
    }

    let mut pos = 0.0;

    let value = if let Some(tp) = last_tp {
        tp.value.unwrap()
    } else { 1.0 };

    if let Some(tp) = iterator.peek() {

        pos = (tp.offset - current_time) * value;

    } else {
        return (time - current_time) * value * scroll_speed;
    }

    while let Some(tp) = iterator.next() {

        let value = tp.value.unwrap();

        if let Some(ntp) = iterator.peek() {
            pos += (ntp.offset - tp.offset) * value;
        } else { // if last
            pos += (time - tp.offset) * value;
            break;
        }
    }
    pos * scroll_speed
}
