extern crate piston;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;


mod remani_window;

use remani_window::RemaniWindow;

fn main() {

    use piston::window::WindowSettings;
    use piston::event_loop::{Events,  EventSettings};
    use piston::input::RenderEvent;
    use glutin_window::GlutinWindow as Window;
    use opengl_graphics::{ OpenGL, GlGraphics };

    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new("Remani", [1024, 768])
                             .opengl(opengl)
                             .srgb(false)
                             .build()
                             .unwrap();

    let mut remani_window = RemaniWindow { gl: GlGraphics::new(opengl) };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            remani_window.render(&r);
        }

    }
}
