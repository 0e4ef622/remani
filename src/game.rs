extern crate piston;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;

use view::View;
use chart::Chart;
use skin::Skin;
use config::Config;

pub fn start(config: Config) {

    use piston::window::WindowSettings;
    use piston::event_loop::{ Events,  EventSettings };
    use piston::input::{ RenderEvent, UpdateEvent, PressEvent };
    use glutin_window::GlutinWindow as Window;
    use opengl_graphics::{ OpenGL, GlGraphics };

    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new("Remani", [1024, 768])
                             .opengl(opengl)
                             .srgb(false)
                             .build()
                             .expect("Could not create window");

    let dpifactor = window.window.hidpi_factor();
    println!("hidpi_factor: {}", dpifactor);

    // test
    let _ = match Chart::from_path("test.osu") {
        Ok(x) => Some(x),
        Err(e) => { println!("{}", e); None },
    };

    let the_skin = Skin::from_path(config.skin_path).unwrap();

    let mut view = View::new(GlGraphics::new(opengl), the_skin);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {

        if let Some(r) = e.render_args() {
            view.render(&r);
        }

        if let Some(u) = e.update_args() {
            view.update(&u);
        }

        if let Some(i) = e.press_args() {
            view.press(&i);
        }

    }

}
