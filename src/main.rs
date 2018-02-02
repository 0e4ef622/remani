extern crate piston;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;

mod view;
mod chart;
mod skin;

use view::View;
use chart::Chart;
use skin::Skin;

fn main() {

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

    let mut view = View::new(GlGraphics::new(opengl), "./test.png");

    // test
    let _ = match Chart::from_path("test.osu") {
        Ok(x) => Some(x),
        Err(e) => { println!("{}", e); None },
    };

    // test
    let _ = match Skin::from_path("test") {
        Ok(x) => Some(x),
        Err(e) => { println!("{}", e); None },
    };

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
