//! Holds the main game logic

use std::time;
use std::path::Path;

use piston;
use opengl_graphics::GlGraphics;
use piston::input::{ RenderEvent, UpdateEvent, PressEvent, ReleaseEvent };

mod model;
mod view;

use chart::Chart;
use config::Config;
use judgement::Judgement;
use self::model::Model;
use self::view::View;
use super::Window;

use audio;
use skin;

pub struct GameScene {
    chart: Chart,
    music: Option<audio::MusicStream<f32>>,
    view: View,
    model: Model,
    time: f64,
    last_instant: time::Instant,
    first_playhead_received: bool,
    first_playhead_request: bool,
}

impl GameScene {
    /// Allocate everything
    pub fn new(chart: Chart, config: &Config, audio: &audio::Audio<f32>) -> GameScene {

        let music = audio::music_from_path(Path::new("test/test_chart").join(&chart.music_path), audio.format()).unwrap();
        let the_skin = skin::from_path(&config.skin_path, config).unwrap();

        let model = Model::new();
        let view = View::new(the_skin);

        GameScene {
            chart,
            music: Some(music),
            view,
            model,
            time: -config.offset,
            last_instant: time::Instant::now(),
            first_playhead_received: false,
            first_playhead_request: false,
        }
    }

    /// Called everytime there is a window event
    pub(super) fn event(&mut self, e: piston::input::Event, config: &Config, audio: &audio::Audio<f32>, window: &mut Window) {

        if self.music.is_some() {
            audio.play_music(self.music.take().unwrap());
        }

        if !self.first_playhead_request {
            audio.request_playhead();
            self.first_playhead_request = true;
        } else if let Some((instant, playhead)) = audio.get_playhead() {

            let d = instant.elapsed();
            let new_time = playhead + d.as_secs() as f64 + d.subsec_nanos() as f64 / 1_000_000_000.0 - config.offset;
            if !self.first_playhead_received {
                self.time = new_time;
                self.first_playhead_received = true;
            } else {
                self.time = (self.time + new_time) / 2.0;
            }
            self.last_instant = time::Instant::now();
            audio.request_playhead();

        } else {

            let d = self.last_instant.elapsed();
            self.time += d.as_secs() as f64 + d.subsec_nanos() as f64 / 1_000_000_000.0;
            self.last_instant = time::Instant::now();

        }

        if let Some(u) = e.update_args() {
            let view = &mut self.view;
            self.model.update(&u, &self.chart, self.time, |k| view.draw_judgement(k, Judgement::Miss));
        }

        if let Some(i) = e.press_args() {
            let view = &mut self.view;
            self.model.press(&i, config, &self.chart, self.time, |k, j| view.draw_judgement(k, j));
        }

        if let Some(i) = e.release_args() {
            self.model.release(&i, config, self.time, |k| ());
        }

        if let Some(r) = e.render_args() {
            self.view.render(&mut window.gl, &r, config, &self.chart, &self.model, self.time);
        }
    }
}
