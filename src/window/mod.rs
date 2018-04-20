//! Handles window creation, holds window and audio handles, and propagates events to Scene structs

use opengl_graphics::GlGraphics;

use config::Config;
use chart::Chart;

use audio;
use skin;

mod game;

enum Scene {
    MainMenu,
    SongSelect,
    Game(game::GameScene),
}

use piston;
impl Scene {
    pub fn event(&mut self, e: piston::input::Event, cfg: &Config, audio: &audio::Audio<f32>, gl: &mut GlGraphics) {
        match *self {
            Scene::Game(ref mut s) => s.event(e, cfg, audio, gl),
            _ => (),
        }
    }
}

struct Stuff {
    current_scene: Scene,
}

pub fn start(config: Config) {
    // oh boy

    use piston::window::WindowSettings;
    use piston::event_loop::{ Events,  EventSettings };
    use glutin_window::GlutinWindow;
    use opengl_graphics::{ OpenGL, GlGraphics };

    let opengl = OpenGL::V3_2;

    let mut window: GlutinWindow = WindowSettings::new("Remani", [1024, 768])
                             .opengl(opengl)
                             .srgb(false)
                             .build()
                             .expect("Could not create window");
    let mut gl = GlGraphics::new(opengl);
    // test
    let chart = match Chart::from_path("test/test_chart/test.osu") {
        Ok(x) => x,
        Err(e) => { println!("{}", e); panic!(); },
    };

    let audio = match audio::start_audio_thread() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return;
        },
    };

    let mut stuff = Stuff {
        current_scene: Scene::Game(game::GameScene::new(chart, &config, &audio)),
    };

    let the_skin = skin::from_path(&config.skin_path, &config).unwrap();

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        stuff.current_scene.event(e, &config, &audio, &mut gl);
    }
}
