use piston::{
    input::{RenderEvent, UpdateEvent},
    window::Window,
};
use texture::CreateTexture;
use conrod_core::{
    Borderable,
    Labelable,
    Positionable,
    Sizeable,
    Widget,
    widget_ids,
};

use super::{game, main_menu::MainMenu, WindowContext};
use crate::{audio, chart, config::Config};

widget_ids! {
    struct Ids {
        list,
        name_text,
        by_text,
        artist_text,
        chart_by_text,
        creator_text,
        diff_list_canvas,
        diff_list,
        back_button,
    }
}

pub struct SongSelect {
    ui: conrod_core::Ui,
    ids: Ids,
    map: conrod_core::image::Map<opengl_graphics::Texture>,
    glyph_cache: conrod_core::text::GlyphCache<'static>,
    glyph_cache_texture: opengl_graphics::Texture,
    song_list: Vec<chart::ChartSet>,
    /// Index into song_list
    selected_song_index: usize,
}

impl SongSelect {
    pub(super) fn new(window_context: &mut WindowContext, _config: &Config) -> Self {
        let song_list = window_context.resources.song_list
            .take()
            .unwrap_or_else(||
                chart::osu::gen_song_list("test")
                .expect("Failed to generate song list"));
        let size = window_context.window.size();
        let mut ui = conrod_core::UiBuilder::new([size.width, size.height]).build();
        ui.handle_event(
            conrod_core::event::Input::Motion(
                conrod_core::input::Motion::MouseCursor {
                    x: window_context.mouse_position[0],
                    y: window_context.mouse_position[1],
                }
            )
        );
        ui.theme.font_id = Some(ui.fonts.insert(window_context.font.clone()));
        ui.theme.shape_color = conrod_core::color::CHARCOAL;
        ui.theme.label_color = conrod_core::color::WHITE;
        let ids = Ids::new(ui.widget_id_generator());
        let map = conrod_core::image::Map::new();
        let glyph_cache = conrod_core::text::GlyphCache::builder()
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
            selected_song_index: window_context.resources.last_selected_song_index, // default is 0
        }
    }
    pub(super) fn event(
        &mut self,
        e: piston::input::Event,
        config: &Config,
        audio: &audio::Audio,
        window_context: &mut WindowContext,
    ) {
        let size = window_context.window.size();
        if let Some(e) = conrod_piston::event::convert(e.clone(), size.width, size.height) {
            self.ui.handle_event(e);
        }
        if let Some(_) = e.update_args() {
            self.set_ui(config, audio, window_context);
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
    fn set_ui(&mut self, config: &Config, audio: &audio::Audio, window_context: &mut WindowContext) {
        let ui = &mut self.ui.set_widgets();

        { // Song list
            let (mut list_items_iter, scrollbar) = conrod_core::widget::List::flow_down(self.song_list.len())
                .middle_of(ui.window)
                .align_right_of(ui.window)
                .item_size(45.0)
                .w(ui.win_w/2.0)
                .kid_area_h_of(ui.window)
                .scrollbar_next_to()
                .set(self.ids.list, ui);

            scrollbar.map(|s| s.set(ui));
            while let Some(item) = list_items_iter.next(ui) {
                let mut button = conrod_core::widget::Button::new()
                    .label(self.song_list[item.i].song_name_unicode
                        .deref()
                        .or(self.song_list[item.i].song_name.deref())
                        .unwrap_or("<UNKNOWN>"))
                    .border(1.0)
                    .border_color(conrod_core::color::WHITE)
                    .label_font_size(15);
                if item.i == self.selected_song_index {
                    button = button.border(2.0).border_color(conrod_core::color::RED);
                }
                if item.set(button, ui).was_clicked() {
                    self.selected_song_index = item.i;
                }
            }
        }

        { // Selected song info
            let selected_song = &self.song_list[self.selected_song_index];
            let song_name = selected_song.song_name_unicode
                .deref()
                .or(selected_song.song_name.deref())
                .unwrap_or("<UNKNOWN>");
            let song_artist = selected_song.artist_unicode
                .deref()
                .or(selected_song.artist.deref())
                .unwrap_or("<UNKNOWN>");

            let chart_creator = selected_song.creator
                .deref()
                .unwrap_or("<UNKNOWN>");

            conrod_core::widget::Text::new(song_name)
                .w(ui.win_w/2.0-50.0)
                .top_left_with_margins_on(ui.window, 50.0, 30.0)
                .font_size(20)
                .set(self.ids.name_text, ui);

            conrod_core::widget::Text::new("Artist: ")
                .down(5.0)
                .font_size(15)
                .set(self.ids.by_text, ui);

            conrod_core::widget::Text::new(song_artist)
                .w(ui.win_w/2.0-50.0-ui.w_of(self.ids.by_text).unwrap_or(0.0))
                .right(0.0)
                .font_size(15)
                .set(self.ids.artist_text, ui);

            conrod_core::widget::Text::new("Chart by: ")
                .top_left_with_margins_on(ui.window, 50.0, 30.0)
                .down(5.0)
                .font_size(15)
                .set(self.ids.chart_by_text, ui);

            conrod_core::widget::Text::new(chart_creator)
                .w(ui.win_w/2.0-50.0-ui.w_of(self.ids.by_text).unwrap_or(0.0))
                .right(0.0)
                .font_size(15)
                .set(self.ids.creator_text, ui);
        }

        { // Current song difficulty list
            let selected_song = &self.song_list[self.selected_song_index];
            let (mut list_items_iter, scrollbar) = conrod_core::widget::List::flow_down(selected_song.difficulties.len())
                .top_left_with_margins_on(ui.window, ui.win_h/2.0, 30.0)
                .item_size(35.0)
                .h(ui.win_h/2.0-30.0)
                .w(ui.win_w/2.0-60.0)
                .scrollbar_on_top()
                .set(self.ids.diff_list, ui);

            scrollbar.map(|s| s.set(ui));
            while let Some(item) = list_items_iter.next(ui) {
                let difficulty = &selected_song.difficulties[item.i];
                let button = conrod_core::widget::Button::new()
                    .label(&difficulty.name)
                    .border(1.0)
                    .border_color(conrod_core::color::WHITE)
                    .label_font_size(15);
                if item.set(button, ui).was_clicked() {
                    match chart::osu::from_path(difficulty.path.clone()) {
                        Ok(x) => Self::change_scene(game::GameScene::new(Box::new(x), config, audio), window_context),
                        Err(e) => println!("{}", e),
                    }
                }
            }
        }

        // back button
        if conrod_core::widget::Button::new()
            .top_left_of(ui.window)
            .w_h(35.0, 25.0)
            .label("back")
            .small_font(&ui)
            .set(self.ids.back_button, ui)
            .was_clicked()
        {
            Self::change_scene(MainMenu::new(), window_context);
        }
    }
    fn change_scene<S: Into<super::Scene> + 'static>(scene: S, window_context: &mut WindowContext) {
        window_context.change_scene_with(move |this: Self, window_context| {
            window_context.resources.song_list = Some(this.song_list);
            window_context.resources.last_selected_song_index = this.selected_song_index;
            scene
        });
    }
}
