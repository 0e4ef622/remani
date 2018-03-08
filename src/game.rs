//! Holds the main game logic

use model::Model;
use view::View;
use chart::Chart;
use config::Config;

use audio;
use skin;

/// Start everything
pub fn start(config: Config) {

    use piston::window::WindowSettings;
    use piston::event_loop::{ Events,  EventSettings };
    use piston::input::{ RenderEvent, UpdateEvent, PressEvent, ReleaseEvent };
    use glutin_window::GlutinWindow as Window;
    use opengl_graphics::{ OpenGL, GlGraphics };

    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new("Remani", [1024, 768])
                             .opengl(opengl)
                             .srgb(false)
                             .build()
                             .expect("Could not create window");

    let dpifactor = window.window.hidpi_factor();
    //println!("hidpi_factor: {}", dpifactor);

    /* test
    let _ = match Chart::from_path("test.osu") {
        Ok(x) => {
            println!("Chart creator:   {}", x.creator.as_ref().unwrap());
            println!("Song artist:     {}", x.artist.as_ref().unwrap());
            println!("Song name:       {}", x.song_name.as_ref().unwrap());
            println!("Difficulty name: {}", x.difficulty_name);
            println!("Timing points:   {}", x.timing_points.len());
            println!("Notes:           {}", x.notes.len());
            Some(x)
        },
        Err(e) => { println!("{}", e); None },
    };
    */

    let audio = audio::start_audio_thread().unwrap();

    let uhhh = audio::music_from_path("test.mp3", audio.format());

    audio.play_music(uhhh);

    let the_skin = skin::from_path(&config.skin_path).unwrap();

    let mut model = Model::new();
    let mut view = View::new(GlGraphics::new(opengl), the_skin);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {

        if let Some(r) = e.render_args() {
            view.render(&r, &model);
        }

        if let Some(u) = e.update_args() {
            model.update(&u);
        }

        if let Some(i) = e.press_args() {
            model.press(&i, &config);
        }

        if let Some(i) = e.release_args() {
            model.release(&i, &config);
        }

    }

}
