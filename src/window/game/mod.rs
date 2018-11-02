//! Holds the main game logic

use std::time;

use opengl_graphics;
use piston::{
    self,
    input::{PressEvent, ReleaseEvent, RenderEvent, UpdateEvent},
};

mod model;
mod view;

use self::{model::Model, view::View};
use super::Window;

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
}

impl GameScene {
    /// Allocate and initialize everything
    pub fn new(mut chart: Box<dyn Chart>, config: &Config, audio: &audio::Audio) -> Self {
        let music = match chart.music(audio.format()) {
            Ok(m) => Some(m),
            Err(e) => {
                remani_warn!("Error loading chart music: {}", e);
                Some(audio::MusicStream::empty())
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
        }
    }

    /// Called everytime there is a window event
    pub(super) fn event(
        &mut self,
        e: piston::input::Event,
        config: &Config,
        audio: &audio::Audio,
        window: &mut Window,
    ) {
        if let Some(m) = self.music.take() {
            audio.play_music(m);
        }

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
            self.model.update(u, config, &*self.chart, self.time, |k| {
                view.draw_judgement(k, Judgement::Miss)
            });
        }

        if let Some(i) = e.press_args() {
            let view = &mut self.view;
            let chart = &*self.chart;
            self.model
                .press(&i, config, chart, self.time, |k, j, note_index| {
                    if let Some(j) = j {
                        view.draw_judgement(k, j);
                    }
                    note_index
                        .and_then(|i| chart.notes()[i].sound_index)
                        .and_then(|i| chart.get_sound(i))
                        .map(|s| audio.play_effect(s));
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
        }
    }
}
