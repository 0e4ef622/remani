//! Handles window creation, holds window and audio handles, and propagates events to Scene structs

use glutin_window::GlutinWindow;
use opengl_graphics::GlGraphics;
use piston::{input::MouseCursorEvent, event_loop::EventLoop};

use crate::{audio, config::Config};

mod game;
mod main_menu;
mod options;

enum Scene {
    MainMenu(main_menu::MainMenu),
    Options(options::Options),
    Game(game::GameScene),
}

impl Scene {
    pub fn event(
        &mut self,
        e: piston::input::Event,
        cfg: &mut Config,
        audio: &audio::Audio,
        window: &mut WindowContext,
    ) {
        match self {
            Scene::Game(scene) => scene.event(e, cfg, audio, window),
            Scene::MainMenu(scene) => scene.event(e, cfg, audio, window),
            Scene::Options(scene) => scene.event(e, cfg, audio, window),
        }
    }
}

impl From<main_menu::MainMenu> for Scene {
    fn from(t: main_menu::MainMenu) -> Self {
        Scene::MainMenu(t)
    }
}

impl From<options::Options> for Scene {
    fn from(t: options::Options) -> Self {
        Scene::Options(t)
    }
}

impl From<game::GameScene> for Scene {
    fn from(t: game::GameScene) -> Self {
        Scene::Game(t)
    }
}

struct WindowContext {
    gl: GlGraphics,
    font: conrod::text::Font, // one font for now

    /// Kept track of here so that mouse coordinates don't get messed up when changing scenes.
    mouse_position: [f64; 2],
    next_scene: Option<Scene>,

    /// The underlying window
    window: GlutinWindow,
}

impl WindowContext {
    fn change_scene<T: Into<Scene>>(&mut self, next_scene: T) {
        self.next_scene = Some(next_scene.into());
    }
}

pub fn start(mut config: Config) {
    use opengl_graphics::{GlGraphics, OpenGL};
    use piston::{
        event_loop::{EventSettings, Events},
        window::WindowSettings,
    };

    let opengl = OpenGL::V3_2;

    let glutin_window: GlutinWindow = WindowSettings::new("Remani", config.general.resolution)
        .opengl(opengl)
        .srgb(false)
        .samples(4)
        .build()
        .expect("Could not create window");
    let gl = GlGraphics::new(opengl);

    let audio = match audio::start_audio_thread(config.general.audio_buffer_size) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let mut window = WindowContext {
        gl,
        next_scene: None,
        font: conrod::text::Font::from_bytes(include_bytes!("../../rsc/fonts/wqy/WenQuanYiMicroHei.ttf") as &[u8])
            .expect("Failed to load Wen Quan Yi Micro Hei font"),
        mouse_position: [-1.0, -1.0],
        window: glutin_window,
    };
    let mut current_scene = Scene::MainMenu(main_menu::MainMenu::new());

    // the UI scenes need to be able to manually swap buffers to allow lazy redrawing
    let mut events = Events::new(EventSettings::new().swap_buffers(false));
    while let Some(e) = events.next(&mut window.window) {
        if let Some(p) = e.mouse_cursor_args() {
            window.mouse_position = p;
        }
        current_scene.event(e, &mut config, &audio, &mut window);

        if window.next_scene.is_some() {
            current_scene = window.next_scene.take().unwrap();
        }
    }
}

// used by conrod
fn cache_glyphs(
    _graphics: &mut opengl_graphics::GlGraphics,
    texture: &mut opengl_graphics::Texture,
    rect: conrod::text::rt::Rect<u32>,
    data: &[u8]
) {
    let mut new_data = Vec::with_capacity((rect.width() * rect.height() * 4) as usize);
    for &a in data {
        new_data.push(255);
        new_data.push(255);
        new_data.push(255);
        new_data.push(a);
    }
    texture::UpdateTexture::update(
        texture,
        &mut (),
        texture::Format::Rgba8,
        &new_data,
        [rect.min.x, rect.min.y],
        [rect.width(), rect.height()],
    ).expect("Error updating glyph cache texture");
}
