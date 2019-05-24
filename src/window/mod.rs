//! Handles window creation, holds window and audio handles, and propagates events to Scene structs

use glutin_window::GlutinWindow;
use opengl_graphics::GlGraphics;
use piston::{input::MouseCursorEvent, event_loop::EventLoop};

use crate::{audio, chart, config::Config};

mod game;
mod main_menu;
mod options;
mod song_select;

enum Scene {
    MainMenu(main_menu::MainMenu),
    Options(options::Options),
    Game(game::GameScene),
    SongSelect(song_select::SongSelect),
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
            Scene::SongSelect(scene) => scene.event(e, cfg, audio, window),
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

impl From<song_select::SongSelect> for Scene {
    fn from(t: song_select::SongSelect) -> Self {
        Scene::SongSelect(t)
    }
}

impl From<Scene> for Option<main_menu::MainMenu> {
    fn from(t: Scene) -> Self {
        match t {
            Scene::MainMenu(s) => Some(s),
            _ => None,
        }
    }
}

impl From<Scene> for Option<options::Options> {
    fn from(t: Scene) -> Self {
        match t {
            Scene::Options(s) => Some(s),
            _ => None,
        }
    }
}

impl From<Scene> for Option<game::GameScene> {
    fn from(t: Scene) -> Self {
        match t {
            Scene::Game(s) => Some(s),
            _ => None,
        }
    }
}

impl From<Scene> for Option<song_select::SongSelect> {
    fn from(t: Scene) -> Self {
        match t {
            Scene::SongSelect(s) => Some(s),
            _ => None,
        }
    }
}

/// A struct for caching things scenes use so e.g. the song list scene doesn't have to regenerate
/// the song list everytime it's viewed.
#[derive(Default)]
struct SceneResources {
    song_list: Option<Vec<chart::ChartSet>>,
    last_selected_song_index: usize,
}

enum NextScene {
    Plain(Scene),
    Function(Box<dyn std::boxed::FnBox(Scene, &mut WindowContext) -> Scene>),
}

struct WindowContext {
    gl: GlGraphics,
    font: conrod_core::text::Font, // one font for now

    /// Kept track of here so that mouse coordinates don't get messed up when changing scenes.
    mouse_position: [f64; 2],
    next_scene: Option<NextScene>,

    /// The underlying window
    window: GlutinWindow,

    resources: SceneResources,
}

impl WindowContext {
    fn change_scene<T: Into<Scene>>(&mut self, next_scene: T) {
        self.next_scene = Some(NextScene::Plain(next_scene.into()));
    }
    fn change_scene_with<S, T, F>(&mut self, next_scene: F)
    where
        Option<S>: From<Scene>,
        T: Into<Scene>,
        F: FnOnce(S, &mut WindowContext) -> T + 'static,
    {
        self.next_scene = Some(NextScene::Function(Box::new(move |scene: Scene, wc: &mut WindowContext| {
            let scene = Option::from(scene).expect("Wrong scene type");
            next_scene(scene, wc).into()
        })));
    }
}

pub fn start(mut config: Config) {
    use opengl_graphics::OpenGL;
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
        font: conrod_core::text::Font::from_bytes(include_bytes!("../../rsc/fonts/wqy/WenQuanYiMicroHei.ttf") as &[u8])
            .expect("Failed to load Wen Quan Yi Micro Hei font"),
        mouse_position: [-1.0, -1.0],
        window: glutin_window,
        resources: SceneResources::default(),
    };
    let mut current_scene = Some(Scene::MainMenu(main_menu::MainMenu::new()));

    // the UI scenes need to be able to manually swap buffers to allow lazy redrawing
    let mut events = Events::new(EventSettings::new().swap_buffers(false));
    while let Some(e) = events.next(&mut window.window) {
        if let Some(p) = e.mouse_cursor_args() {
            window.mouse_position = p;
        }
        current_scene.as_mut().unwrap().event(e, &mut config, &audio, &mut window);

        if window.next_scene.is_some() {
            current_scene = Some(match window.next_scene.take() {
                Some(NextScene::Plain(s)) => s,
                Some(NextScene::Function(f)) => f.call_box((current_scene.take().unwrap(), &mut window)),
                None => unreachable!(),
            });
        }
    }
}

// used by conrod
fn cache_glyphs(
    _graphics: &mut opengl_graphics::GlGraphics,
    texture: &mut opengl_graphics::Texture,
    rect: conrod_core::text::rt::Rect<u32>,
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
