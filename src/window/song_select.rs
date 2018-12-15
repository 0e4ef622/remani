#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use piston::{
    input::{self, ButtonEvent, PressEvent, RenderEvent, UpdateEvent},
    window::Window,
};
use texture::CreateTexture;
use conrod::{
    backend::piston as conrod_piston,
    Borderable,
    Colorable,
    Labelable,
    Positionable,
    Sizeable,
    Widget,
    widget_ids,
};

use super::WindowContext;
use crate::{audio, chart, config::Config};

widget_ids! {
    struct Ids {
        list,
        name_text,
        diff_name_text,
        artist_text,
        creator_text,
    }
}

pub struct SongSelect {
    ui: conrod::Ui,
    ids: Ids,
    map: conrod::image::Map<opengl_graphics::Texture>,
    glyph_cache: conrod::text::GlyphCache<'static>,
    glyph_cache_texture: opengl_graphics::Texture,
    song_list: Vec<chart::ChartSet>,
}

impl SongSelect {
    pub(super) fn new(window_context: &WindowContext, config: &Config) -> Self {
        let song_list = chart::osu::gen_song_list("test").unwrap();
        let size = window_context.window.size();
        let mut ui = conrod::UiBuilder::new([size.width, size.height]).build();
        ui.handle_event(
            conrod::event::Input::Motion(
                conrod::input::Motion::MouseCursor {
                    x: window_context.mouse_position[0],
                    y: window_context.mouse_position[1],
                }
            )
        );
        ui.theme.font_id = Some(ui.fonts.insert(window_context.font.clone()));
        ui.theme.shape_color = conrod::color::CHARCOAL;
        ui.theme.label_color = conrod::color::WHITE;
        let ids = Ids::new(ui.widget_id_generator());
        let map = conrod::image::Map::new();
        let glyph_cache = conrod::text::GlyphCache::builder()
            .dimensions(1024, 1024)
            .build();
        let vec = vec![0; 1024*1024*4];
        let glyph_cache_texture = opengl_graphics::Texture::create(
            &mut (),
            texture::Format::Rgba8,
            &vec,
            [1024, 1024],
            &texture::TextureSettings::new(),
        ).expect("failed to create texture");
        Self {
            ui,
            ids,
            map,
            glyph_cache,
            glyph_cache_texture,
            song_list,
        }
    }
    pub(super) fn event(
        &mut self,
        e: piston::input::Event,
        config: &Config,
        _audio: &audio::Audio,
        window_context: &mut WindowContext,
    ) {
        let size = window_context.window.size();
        if let Some(e) = conrod_piston::event::convert(e.clone(), size.width, size.height) {
            self.ui.handle_event(e);
        }
        if let Some(_) = e.update_args() {
            self.set_ui(config, window_context);
        }
        if let Some(r) = e.render_args() {
            if let Some(primitives) = self.ui.draw_if_changed() {
                let self_glyph_cache_texture = &mut self.glyph_cache_texture;
                let self_glyph_cache = &mut self.glyph_cache;
                let self_map = &self.map;
                window_context.gl.draw(r.viewport(), |c, gl| {
                    graphics::clear([0.0, 0.0, 0.0, 1.0], gl);
                    conrod_piston::draw::primitives(
                        primitives,
                        c,
                        gl,
                        self_glyph_cache_texture,
                        self_glyph_cache,
                        self_map,
                        super::cache_glyphs,
                        |t| t,
                    );
                });
                window_context.window.swap_buffers();
            }
        }
    }
    fn set_ui(&mut self, config: &Config, window_context: &mut WindowContext) {
        let ui = &mut self.ui.set_widgets();

        let (mut list_items_iter, scrollbar) = conrod::widget::List::flow_down(self.song_list.len())
            .middle_of(ui.window)
            .align_right_of(ui.window)
            .item_size(40.0)
            .w(ui.win_w/2.0)
            .kid_area_h_of(ui.window)
            .scrollbar_next_to()
            .set(self.ids.list, ui);

        scrollbar.map(|s| s.set(ui));
        while let Some(item) = list_items_iter.next(ui) {
            item.set(
                conrod::widget::Button::new()
                    .label(self.song_list[item.i].song_name.as_ref().map(|s| &**s).unwrap_or("UNNAMED"))
                    .border(1.0)
                    .border_color(conrod::color::WHITE),
                    // .small_font(ui),
                ui
            );
        }

        conrod::widget::Text::new("Song name")
            .font_size(ui.theme().font_size_small)
            .top_left_with_margins_on(ui.window, 70.0, 30.0)
            .font_size(18)
            .set(self.ids.name_text, ui);

        conrod::widget::Text::new("Difficulty name")
            .font_size(ui.theme().font_size_small)
            .down(30.0)
            .font_size(18)
            .set(self.ids.diff_name_text, ui);

        conrod::widget::Text::new("Song artist")
            .font_size(ui.theme().font_size_small)
            .down(30.0)
            .font_size(18)
            .set(self.ids.artist_text, ui);

        conrod::widget::Text::new("Chart creator")
            .font_size(ui.theme().font_size_small)
            .down(30.0)
            .font_size(18)
            .set(self.ids.creator_text, ui);
    }
}
