use graphics::{self, draw_state::DrawState, image::Image};
use opengl_graphics::Texture;
use piston::{
    self,
    input::{mouse, Button, MouseCursorEvent, PressEvent, RenderEvent},
};
use texture::{ImageSize, TextureSettings};

use super::{game, Scene, Window};
use crate::{audio, chart::Chart, config::Config};

pub struct MainMenu {
    play_texture: Texture,
    options_texture: Texture,
    exit_texture: Texture,
    mouse_position: [f64; 2],
    window_height: f64,
}

impl MainMenu {
    pub fn new() -> MainMenu {
        let ts = TextureSettings::new();

        let play_texture = Texture::from_path("rsc/play.png", &ts).unwrap();
        let options_texture = Texture::from_path("rsc/options.png", &ts).unwrap();
        let exit_texture = Texture::from_path("rsc/exit.png", &ts).unwrap();

        MainMenu {
            play_texture,
            options_texture,
            exit_texture,
            mouse_position: [-1.0, -1.0],
            window_height: 0.0,
        }
    }

    /// Called everytime there is a window event
    pub(super) fn event(
        &mut self,
        e: piston::input::Event,
        config: &Config,
        audio: &audio::Audio<f32>,
        window: &mut Window,
    ) {
        if let Some(p) = e.mouse_cursor_args() {
            self.mouse_position = p;
        }

        if let Some(i) = e.press_args() {
            if i == Button::Mouse(mouse::MouseButton::Left) && self.mouse_position[1] < self.window_height / 3.0 {
                let chart = match Chart::from_path("test/test_chart/test.osu") {
                    Ok(x) => x,
                    Err(e) => {
                        println!("{}", e);
                        panic!();
                    }
                };
                window.change_scene(Scene::Game(game::GameScene::new(chart, config, audio)));
            }
        }

        // if let Some(i) = e.release_args() {
        // }

        if let Some(r) = e.render_args() {
            window.gl.draw(r.viewport(), |c, gl| {
                self.window_height = r.height as f64;

                let play_texture = &self.play_texture;
                let options_texture = &self.options_texture;
                let exit_texture = &self.exit_texture;

                graphics::clear([0.0, 0.0, 0.0, 0.0], gl);

                let draw_state = DrawState::default();

                let play_w = play_texture.get_width() as f64;
                let play_h = play_texture.get_height() as f64;
                let play_x = r.width as f64 / 2.0 - play_w / 2.0;
                let play_y = r.height as f64 / 4.0 - play_h / 2.0;;
                let play_image = Image::new().rect([play_x, play_y, play_w, play_h]);

                let options_w = options_texture.get_width() as f64;
                let options_h = options_texture.get_height() as f64;
                let options_x = r.width as f64 / 2.0 - options_w / 2.0;
                let options_y = r.height as f64 / 2.0 - options_h / 2.0;;
                let options_image = Image::new().rect([options_x, options_y, options_w, options_h]);

                let exit_w = exit_texture.get_width() as f64;
                let exit_h = exit_texture.get_height() as f64;
                let exit_x = r.width as f64 / 2.0 - exit_w / 2.0;
                let exit_y = 3.0 * r.height as f64 / 4.0 - exit_h / 2.0;;
                let exit_image = Image::new().rect([exit_x, exit_y, exit_w, exit_h]);

                options_image.draw(options_texture, &draw_state, c.transform, gl);
                exit_image.draw(exit_texture, &draw_state, c.transform, gl);
                play_image.draw(play_texture, &draw_state, c.transform, gl);
            });
        }
    }
}
