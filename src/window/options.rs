use piston::{
    input::{UpdateEvent, RenderEvent},
    window::Window as __,
};
use texture::CreateTexture;
use conrod::{
    backend::piston as conrod_piston,
    Positionable,
    Sizeable,
    Widget,
    widget_ids,
};

// use super::{main_menu::MainMenu, Window};
use super::Window;
use crate::{audio, config::Config};

widget_ids! {
    struct Ids {
        button,
    }
}

pub struct Options {
    ui: conrod::Ui,
    ids: Ids,
    map: conrod::image::Map<opengl_graphics::Texture>,
    glyph_cache: conrod::text::GlyphCache<'static>,
    glyph_cache_texture: opengl_graphics::Texture,
}

impl Options {
    pub(super) fn new(window: &Window) -> Self {
        let size = window.window.size();
        let mut ui = conrod::UiBuilder::new([size.width, size.height]).build();
        ui.handle_event(conrod::event::Input::Motion(conrod::input::Motion::MouseCursor { x: window.mouse_position[0], y: window.mouse_position[1] }));
        // ui.theme.font_id = Some(ui.fonts.insert(window.glyph_cache.font.clone()));
        let ids = Ids::new(ui.widget_id_generator());
        let map = conrod::image::Map::new();
        let glyph_cache = conrod::text::GlyphCache::builder()
            .dimensions(1, 1)
            .build();
        let glyph_cache_texture = opengl_graphics::Texture::create(
            &mut (),
            texture::Format::Rgba8,
            &[0; 1*1],
            [1, 1], // TODO
            &texture::TextureSettings::new(),
        ).expect("failed to create texture");
        Self { ui, ids, map, glyph_cache, glyph_cache_texture }
    }
    pub(super) fn event(
        &mut self,
        e: piston::input::Event,
        _config: &Config,
        _audio: &audio::Audio,
        window: &mut Window,
    ) {
        let size = window.window.size();
        if let Some(e) = conrod_piston::event::convert(e.clone(), size.width, size.height) {
            self.ui.handle_event(e);
        }
        if let Some(_) = e.update_args() {
            let mut ui = self.ui.set_widgets();
            if conrod::widget::Button::new()
                .x_y_relative(0.0, 0.0)
                .w_h(50.0, 50.0)
                .set(self.ids.button, &mut ui)
                .was_clicked()
            {
                    println!("button clicked!");
            }

        }
        if let Some(r) = e.render_args() {
            window.gl.draw(r.viewport(), |c, gl| {
                graphics::clear([0.0, 0.0, 0.0, 1.0], gl);
                if let Some(primitives) = Some(self.ui.draw()) {
                    conrod_piston::draw::primitives(
                        primitives,
                        c,
                        gl,
                        &mut self.glyph_cache_texture,
                        &mut self.glyph_cache,
                        &self.map,
                        |_,_,_,_| (),
                        |t| t,
                    );
                }
            });
        }
    }
}
