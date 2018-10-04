//! A module that handles window update and key press/release events

use std::collections::VecDeque;

use piston::input::{Button, UpdateArgs};

use crate::{chart::Chart, config::Config, judgement::Judgement};

/// Holds game states needed by the logic and renderer. Also does timing judgements.
pub struct Model {
    pub keys_down: [bool; 7],

    /// Contains the index of the first note that is 1 second ahead of the current time.
    current_note_index: usize,

    /// Contains the indices of all the notes that are to be hit within the next second or haven't
    /// been hit yet (1 second into the future, 1 second into the past), categorized into which
    /// column they are on.
    next_notes: [VecDeque<usize>; 7],

    /// Whether the column is currently holding a long note, and if so, contains the index of the
    /// note
    long_notes_held: [Option<usize>; 7],
}

impl Model {
    /// Create a model for the game controller
    pub fn new() -> Model {
        Model {
            keys_down: [false; 7],
            current_note_index: 0,
            next_notes: [
                VecDeque::with_capacity(32),
                VecDeque::with_capacity(32),
                VecDeque::with_capacity(32),
                VecDeque::with_capacity(32),
                VecDeque::with_capacity(32),
                VecDeque::with_capacity(32),
                VecDeque::with_capacity(32),
            ],
            long_notes_held: [None; 7],
        }
    }

    /// Called when an update event occurs
    pub fn update<F: FnMut(usize)>(
        &mut self,
        _args: UpdateArgs,
        config: &Config,
        chart: &Chart,
        time: f64,
        mut miss_callback: F,
    ) {
        // how many notes should be removed from the front of each vecdeque
        let mut to_be_removed = [0; 7];

        for (column, note_vec) in self.next_notes.iter().enumerate() {
            for &note_index in note_vec {
                let note = &chart.notes[note_index];
                if let Some(end_time) = note.end_time {
                    // TODO dont hardcode timing windows
                    if end_time - time < -0.3 {
                        miss_callback(note_index);
                        to_be_removed[column] += 1;
                    }
                } else if note.time - time < -0.3 {
                    miss_callback(note_index);
                    to_be_removed[column] += 1;
                }
            }
        }

        for (column, &n) in to_be_removed.iter().enumerate() {
            for _ in 0..n {
                self.next_notes[column].pop_front();
            }
        }

        while chart.notes.get(self.current_note_index)
            .map(|n| n.time - time < config.game.current_judge().1.miss_tolerance)
            .unwrap_or(false) {

            self.next_notes[chart.notes[self.current_note_index].column]
                .push_back(self.current_note_index);
            self.current_note_index += 1;
        }
    }

    /// Called when a press event occurs
    pub fn press<F: FnMut(usize, Option<Judgement>)>(
        &mut self,
        args: &Button,
        config: &Config,
        chart: &Chart,
        time: f64,
        mut callback: F,
    ) {
        let next_notes = &mut self.next_notes;
        let long_notes_held = &mut self.long_notes_held;

        config.game.key_bindings
            .iter()
            .enumerate()
            .zip(self.keys_down.iter_mut())
            .for_each(|((key_index, key_binding), key_down)| {
                if *args == *key_binding && !*key_down {
                    let judgement = if let Some(&note_index) = next_notes[key_index].get(0) {
                        let note = &chart.notes[note_index];
                        next_notes[key_index].pop_front();

                        let timing = note.time - time;
                        if note.end_time.is_some() {
                            println!("LN {} press: {:+.3}", key_index, timing);
                            debug_assert_eq!(long_notes_held[key_index], None);
                            long_notes_held[key_index] = Some(note_index);
                        }

                        // TODO dont hardcode timing windows
                        if timing.abs() < 0.1 {
                            Some(Judgement::Perfect)
                        } else {
                            Some(Judgement::Miss)
                        }
                    } else {
                        None
                    };

                    *key_down = true;

                    callback(key_index, judgement);
                }
            });
    }

    pub fn release<F: FnMut(usize)>(
        &mut self,
        args: &Button,
        config: &Config,
        chart: &Chart,
        time: f64,
        mut callback: F,
    ) {
        let long_notes_held = &mut self.long_notes_held;
        config.game.key_bindings
            .iter()
            .enumerate()
            .zip(self.keys_down.iter_mut())
            .for_each(|((key_index, key_binding), key_down)| {
                if *args == *key_binding {
                    callback(key_index);
                    *key_down = false;
                    if let Some(note_index) = long_notes_held[key_index] {
                        println!("LN {} release: {:+.3}", key_index, chart.notes[note_index].end_time.unwrap() - time);
                        long_notes_held[key_index] = None;
                    }
                }
            });
    }
}
