//! Holds the main game logic

use std::time;

use piston::{
    self,
    input::{PressEvent, ReleaseEvent, RenderEvent, UpdateEvent},
    window::Window,
};

mod model;
mod view;

use self::{model::Model, view::View};
use super::{song_select::SongSelect, WindowContext};

use crate::{audio, chart::Chart, config::Config, judgement::Judgement, gameskin};

pub struct GameScene {
    chart: Box<dyn Chart>,
    music: Option<audio::MusicStream>,
    view: View<opengl_graphics::GlGraphics>,
    model: Model,
    time: f64,
    last_instant: time::Instant,
    first_playhead_received: bool,
    first_playhead_request: bool,
    current_autoplay_sound_index: usize,
    chart_end_time: Option<f64>,
}

impl GameScene {
    /// Allocate and initialize everything
    pub fn new(mut chart: Box<dyn Chart>, config: &Config, audio: &audio::Audio) -> Self {
        let music = match chart.music(audio.format()) {
            Ok(m) => Some(m),
            Err(e) => {
                remani_warn!("Error loading chart music `{}'", e);
                Some(audio::MusicStream::zero())
            }
        };
        chart.load_sounds(audio.format(), config);
        let the_skin = gameskin::from_path(&mut (), &config.game.current_skin().1, config).unwrap();

        let model = Model::new();
        let view = View::new(the_skin);

        GameScene {
            chart,
            music,
            view,
            model,
            time: config.game.offset,
            last_instant: time::Instant::now(),
            first_playhead_received: false,
            first_playhead_request: false,
            current_autoplay_sound_index: 0,
            chart_end_time: None,
        }
    }

    /// Called everytime there is a window event
    pub(super) fn event(
        &mut self,
        e: piston::input::Event,
        config: &Config,
        audio: &audio::Audio,
        window: &mut WindowContext,
    ) {
        self.music.take()
            .map(|m| audio.play_music(m) || panic!("Failed to play music"));

        if !self.first_playhead_request {
            if let Err(e) = audio.request_playhead() {
                remani_warn!("Error requesting audio playhead: {}", e);
            }
            self.first_playhead_request = true;
        } else if let Some((instant, playhead)) = audio.get_playhead() {
            let d = instant.elapsed();
            let new_time =
                playhead + d.as_secs() as f64 + d.subsec_nanos() as f64 / 1e9
                    + config.game.offset;
            if !self.first_playhead_received {
                self.time = new_time;
                self.first_playhead_received = true;
            } else {
                self.time = (self.time + new_time) / 2.0;
            }
            self.last_instant = time::Instant::now();

            if let Err(e) = audio.request_playhead() {
                remani_warn!("Error requesting audio playhead: {}", e);
            }
        } else {
            let d = self.last_instant.elapsed();
            self.time += d.as_secs() as f64 + d.subsec_nanos() as f64 / 1e9;
            self.last_instant = time::Instant::now();
        }

        if let Some(u) = e.update_args() {
            let view = &mut self.view;
            // Update notes in model, draw any misses that occurred
            self.model.update(u, config, &*self.chart, self.time, |k| {
                view.draw_judgement(k, Judgement::Miss, false)
            });
            // Play the autoplay sounds if one needs to be played
            if let Some(autoplay_sound) = self.chart.autoplay_sounds().get(self.current_autoplay_sound_index) {
                // Unapply the offset to make sure the autoplay sound lines up with the music
                if self.time - config.game.offset >= autoplay_sound.time {
                    self.current_autoplay_sound_index += 1;
                    self.chart
                        .get_sound(autoplay_sound.sound_index)
                        .map(|s|
                            audio.play_effect(s.with_volume(autoplay_sound.volume))
                            || panic!("Failed to play effect")
                        );
                }
            }
            if view.chart_ended(&*self.chart) && self.chart_end_time.is_none() {
                self.chart_end_time = Some(self.time);
            }

            if let Some(chart_end_time) = self.chart_end_time {
                if self.time - 10.0 > chart_end_time {
                    let song_select_scene = SongSelect::new(window, config);
                    window.change_scene(song_select_scene);
                }
            }
        }

        if let Some(i) = e.press_args() {
            let view = &mut self.view;
            let chart = &*self.chart;
            self.model
                .press(&i, config, chart, self.time, |k, j, note_index, is_long_note| {
                    if let Some(j) = j {
                        view.draw_judgement(k, j, is_long_note);
                    }
                    note_index
                        .and_then(|i| chart.notes()[i].sound_index)
                        .and_then(|i| chart.get_sound(i))
                        .map(|s| audio.play_effect(s) || panic!("Failed to play effect"));
                    view.key_down(k);
                });
        }

        if let Some(i) = e.release_args() {
            let view = &mut self.view;
            self.model
                .release(&i, config, &*self.chart, self.time, |k| view.key_up(k));
        }

        if let Some(r) = e.render_args() {
            window.gl.draw(r.viewport(), |c, mut gl| {
                self.view
                    .render(c, &mut gl, &r, config, &*self.chart, &self.model, self.time);
            });
            window.window.swap_buffers();
        }
    }
}
