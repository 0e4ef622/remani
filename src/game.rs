//! Holds the main game logic

use std::time;
use std::path::Path;

use model::{ Model, Judgement };
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

    // test
    let chart = match Chart::from_path("test/test_chart/test.osu") {
        Ok(x) => x,
        Err(e) => { println!("{}", e); panic!(); },
    };


    let audio = audio::start_audio_thread().unwrap();

    let uhhh = audio::music_from_path(Path::new("test/test_chart").join(&chart.music_path), audio.format()).unwrap();

    audio.play_music(uhhh);

    let the_skin = skin::from_path(&config.skin_path, &config).unwrap();

    let mut model = Model::new(&chart, &config);
    let mut view = View::new(GlGraphics::new(opengl), the_skin, &chart);

    let mut events = Events::new(EventSettings::new());
    let mut time = -config.offset;
    let mut last_instant = time::Instant::now();
    let mut first_playhead_received = false;
    audio.request_playhead();
    while let Some(e) = events.next(&mut window) {

        if let Some((instant, playhead)) = audio.get_playhead() {

            let d = instant.elapsed();
            let new_time = playhead + d.as_secs() as f64 + d.subsec_nanos() as f64 / 1_000_000_000.0 - config.offset;
            if !first_playhead_received {
                time = new_time;
            } else {
                time = (time + new_time) / 2.0;
            }
            last_instant = time::Instant::now();
            audio.request_playhead();

        } else {

            let d = last_instant.elapsed();
            time += d.as_secs() as f64 + d.subsec_nanos() as f64 / 1_000_000_000.0;
            last_instant = time::Instant::now();

        }

        if let Some(u) = e.update_args() {
            model.update(&u, time, |k| view.draw_judgement(k, Judgement::Miss));
        }

        if let Some(i) = e.press_args() {
            model.press(&i, time, |k, j| view.draw_judgement(k, j));
        }

        if let Some(i) = e.release_args() {
            model.release(&i, time, |k| ());
        }

        if let Some(r) = e.render_args() {
            view.render(&r, &config, &model, time);
        }
    }
}
