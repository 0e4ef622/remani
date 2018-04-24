//! Handles window creation, holds window and audio handles, and propagates events to Scene structs

use opengl_graphics::GlGraphics;

use config::Config;
use chart::Chart;

use audio;

mod game;
mod main_menu;

enum Scene {
    MainMenu(main_menu::MainMenu),
    SongSelect,
    Game(game::GameScene),
}

use piston;
impl Scene {
    pub fn event(&mut self, e: piston::input::Event, cfg: &Config, audio: &audio::Audio<f32>, window: &mut Window) {
        match *self {
            Scene::Game(ref mut scene)     => scene.event(e, cfg, audio, window),
            Scene::MainMenu(ref mut scene) => scene.event(e, cfg, audio, window),
            _ => (),
        }
    }
}

struct Window {
    gl: GlGraphics,
    next_scene: Option<Scene>,
}

impl Window {
    fn change_scene(&mut self, next_scene: Scene) {
        self.next_scene = Some(next_scene);
    }
}

pub fn start(config: Config) {

    use piston::window::WindowSettings;
    use piston::event_loop::{ Events,  EventSettings };
    use glutin_window::GlutinWindow;
    use opengl_graphics::{ OpenGL, GlGraphics };

    let opengl = OpenGL::V3_2;

    let mut glutin_window: GlutinWindow = WindowSettings::new("Remani", [1024, 768])
                             .opengl(opengl)
                             .srgb(false)
                             .build()
                             .expect("Could not create window");
    let mut gl = GlGraphics::new(opengl);

    let audio = match audio::start_audio_thread() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return;
        },
    };

    let mut window = Window {
        gl,
        next_scene: None,
    };
    let mut current_scene = Scene::MainMenu(main_menu::MainMenu::new());

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut glutin_window) {
        current_scene.event(e, &config, &audio, &mut window);

        if window.next_scene.is_some() {
            current_scene = window.next_scene.take().unwrap();
        }
    }
}