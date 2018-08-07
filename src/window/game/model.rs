//! A module that handles window update and key press/release events

use std::collections::VecDeque;

use piston::input::{Button, UpdateArgs};

use crate::chart::Chart;
use crate::config::Config;
use crate::judgement::Judgement;

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
        _args: &UpdateArgs,
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

        while self.current_note_index < chart.notes.len()
            && chart.notes[self.current_note_index].time - time < 1.0
        {
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

        config
            .key_bindings
            .iter()
            .enumerate()
            .zip(self.keys_down.iter_mut())
            .for_each(|((key_index, key_binding), key_down)| {
                if *args == *key_binding && !*key_down {
                    let judgement = if let Some(&note_index) = next_notes[key_index].get(0) {
                        let note = &chart.notes[note_index];
                        next_notes[key_index].pop_front();

                        // TODO dont hardcode timing windows
                        if (note.time - time).abs() < 0.1 {
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
        _time: f64,
        mut callback: F,
    ) {
        config
            .key_bindings
            .iter()
            .enumerate()
            .zip(self.keys_down.iter_mut())
            .for_each(|((key_index, key_binding), key_down)| {
                if *args == *key_binding {
                    callback(key_index);
                    *key_down = false;
                }
            });
    }
}
