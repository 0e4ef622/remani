use piston::{
    input::{UpdateEvent, RenderEvent},
    window::Window as __,
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

use super::{main_menu::MainMenu, WindowContext};
use crate::{audio, config::Config};

use std::fmt::Write;

widget_ids! {
    struct Ids {
        main_canvas,
        main_scrollbar,
        win_res_text,
        win_res_x,
        win_res_canvas,
        win_res_w,
        win_res_h,
        back_button,
        button,
        slider,
        textbox,
        toggle,
        toggle_canvas,
        toggle_label,
        list,
    }
}

pub struct Options {
    ui: conrod::Ui,
    ids: Ids,
    map: conrod::image::Map<opengl_graphics::Texture>,
    glyph_cache: conrod::text::GlyphCache<'static>,
    glyph_cache_texture: opengl_graphics::Texture,
    slider_value: f32,
    slider_label: String,
    win_res_w_text: String,
    win_res_h_text: String,
    toggle_value: bool,
}

impl Options {
    pub(super) fn new(window: &WindowContext) -> Self {
        let size = window.window.size();
        let mut ui = conrod::UiBuilder::new([size.width, size.height]).build();
        ui.handle_event(conrod::event::Input::Motion(conrod::input::Motion::MouseCursor { x: window.mouse_position[0], y: window.mouse_position[1] }));
        ui.theme.font_id = Some(ui.fonts.insert(window.glyph_cache.font.clone()));
        ui.theme.shape_color = conrod::color::CHARCOAL;
        // ui.theme.border_color = conrod::color::Rgba(1.0, 1.0, 1.0, 1.0).into();
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
        let slider_value = 0.0;
        let slider_label = String::new();
        let win_res_w_text = String::from("1024");
        let win_res_h_text = String::from("768");
        let toggle_value = true;
        Self {
            ui,
            ids,
            map,
            glyph_cache,
            glyph_cache_texture,
            slider_value,
            slider_label,
            win_res_w_text,
            win_res_h_text,
            toggle_value
        }
    }
    pub(super) fn event(
        &mut self,
        e: piston::input::Event,
        _config: &mut Config,
        _audio: &audio::Audio,
        window_context: &mut WindowContext,
    ) {
        let size = window_context.window.size();
        if let Some(e) = conrod_piston::event::convert(e.clone(), size.width, size.height) {
            self.ui.handle_event(e);
        }
        if let Some(_) = e.update_args() {
            // Set the UI
            self.set_ui(window_context);
        }
        if let Some(r) = e.render_args() {
            window_context.gl.draw(r.viewport(), |c, gl| {
                graphics::clear([0.0, 0.0, 0.0, 1.0], gl);
                if let Some(primitives) = Some(self.ui.draw()) {
                    conrod_piston::draw::primitives(
                        primitives,
                        c,
                        gl,
                        &mut self.glyph_cache_texture,
                        &mut self.glyph_cache,
                        &self.map,
                        cache_glyphs,
                        |t| t,
                    );
                }
            });
        }
    }
    fn set_ui(&mut self, window_context: &mut WindowContext) {
        let ui = &mut self.ui.set_widgets();

        conrod::widget::Canvas::new()
            .w(500.0)
            .border(0.0)
            .pad(50.0)
            .set(self.ids.main_canvas, ui);

        conrod::widget::Scrollbar::y_axis(self.ids.main_canvas)
            .auto_hide(true)
            .set(self.ids.main_scrollbar, ui);

        // test button
        if conrod::widget::Button::new()
            .top_right_of(self.ids.main_canvas)
            .w_h(30.0, 20.0)
            .label("test")
            .small_font(&ui)
            .set(self.ids.button, ui)
            .was_clicked()
        {
            println!("button clicked!");
        }

        // // slider thing
        // let slider_value = &mut self.slider_value;
        // self.slider_label.clear();
        // write!(self.slider_label, "{:.2}", slider_value).expect("wtf");
        // conrod::widget::Slider::new(*slider_value, -1.0, 1.0)
        //     .w_h(300.0, 20.0)
        //     .align_right()
        //     .label(&self.slider_label)
        //     .small_font(&ui)
        //     .set(self.ids.slider, ui)
        //     .map(|v| *slider_value = v);

        conrod::widget::Canvas::new()
            .kid_area_w_of(self.ids.main_canvas)
            .h(20.0)
            .align_right()
            .down(20.0)
            .border(0.0)
            .set(self.ids.win_res_canvas, ui);

        // Text description
        conrod::widget::Text::new("Window resolution")
            .font_size(ui.theme().font_size_small)
            .top_left_of(self.ids.win_res_canvas)
            .set(self.ids.win_res_text, ui);

        // height field
        let self_win_res_h_text = &mut self.win_res_h_text;
        conrod::widget::TextBox::new(self_win_res_h_text)
            .font_size(ui.theme().font_size_small)
            .w_h(50.0, 20.0)
            .top_right_of(self.ids.win_res_canvas)
            .border_color(conrod::color::WHITE)
            .set(self.ids.win_res_h, ui)
            .into_iter()
            .fold(None, |a, e| if let conrod::widget::text_box::Event::Update(s) = e { Some(s) } else { a })
            .map(|s| *self_win_res_h_text = s);

        // The "x" in between
        conrod::widget::Text::new("x")
            .font_size(ui.theme().font_size_small)
            .left(5.0)
            .set(self.ids.win_res_x, ui);

        // width field
        let self_win_res_w_text = &mut self.win_res_w_text;
        conrod::widget::TextBox::new(self_win_res_w_text)
            .font_size(ui.theme().font_size_small)
            .w_h(50.0, 20.0)
            .left(5.0)
            .border_color(conrod::color::WHITE)
            .set(self.ids.win_res_w, ui)
            .into_iter()
            .fold(None, |a, e| if let conrod::widget::text_box::Event::Update(s) = e { Some(s) } else { a })
            .map(|s| *self_win_res_w_text = s);

        // togglerino
        conrod::widget::Canvas::new()
            .w_h(300.0, 20.0)
            .align_right_of(self.ids.win_res_h)
            .y_position(ui.theme.y_position)
            .rgb(1.0, 0.0, 0.0)
            .set(self.ids.toggle_canvas, ui);
        let self_toggle_value = &mut self.toggle_value;
        conrod::widget::toggle::Toggle::new(*self_toggle_value)
            .w_h(20.0, 20.0)
            .top_right_of(self.ids.toggle_canvas)
            .label("thingo")
            .small_font(&ui)
            .set(self.ids.toggle, ui)
            .last()
            .map(|v| *self_toggle_value = v);

        // back button
        if conrod::widget::Button::new()
            .top_left_of(ui.window)
            .w_h(35.0, 25.0)
            .label("back")
            .small_font(&ui)
            .set(self.ids.back_button, ui)
            .was_clicked()
        {
            window_context.change_scene(MainMenu::new());
        }
    }
}

fn cache_glyphs(
    _graphics: &mut opengl_graphics::GlGraphics,
    texture: &mut opengl_graphics::Texture,
    rect: conrod::text::rt::Rect<u32>,
    data: &[u8]
) {
    let mut new_data = Vec::with_capacity((rect.width() * rect.height() * 4) as usize);
    for &a in data {
        new_data.push(255);
        new_data.push(255);
        new_data.push(255);
        new_data.push(a);
    }
    opengl_graphics::UpdateTexture::update(
        texture,
        &mut (),
        texture::Format::Rgba8,
        &new_data,
        [rect.min.x, rect.min.y],
        [rect.width(), rect.height()],
    ).expect("Error updating glyph cache texture");
}
