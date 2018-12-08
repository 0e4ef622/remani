use piston::{
    input::{self, ButtonEvent, PressEvent, RenderEvent, UpdateEvent},
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
use crate::{audio, config::{self, Config}};

widget_ids! {
    struct Ids {
        main_canvas,
        main_scrollbar,
        win_res_text,
        win_res_x_text,
        win_res_canvas,
        win_res_w_input,
        win_res_h_input,
        audio_buf_size_canvas,
        audio_buf_size_text,
        audio_buf_size_input,
        audio_offset_canvas,
        audio_offset_text,
        audio_offset_input,
        scroll_speed_canvas,
        scroll_speed_text,
        scroll_speed_input,
        enable_osu_hit_sounds_canvas,
        enable_osu_hit_sounds_text,
        enable_osu_hit_sounds_toggle,
        keybindings_canvas,
        keybindings_text,
        keybindings_buttons_canvas,
        key0_canvas,
        key1_canvas,
        key2_canvas,
        key3_canvas,
        key4_canvas,
        key5_canvas,
        key6_canvas,
        key0_button,
        key1_button,
        key2_button,
        key3_button,
        key4_button,
        key5_button,
        key6_button,
        back_button,
    }
}

pub struct Options {
    ui: conrod::Ui,
    ids: Ids,
    map: conrod::image::Map<opengl_graphics::Texture>,
    glyph_cache: conrod::text::GlyphCache<'static>,
    glyph_cache_texture: opengl_graphics::Texture,
    win_res_w_text: String,
    win_res_h_text: String,
    audio_buf_size_input_text: String,
    audio_offset_input_text: String,
    scroll_speed_input_text: String,
    enable_osu_hit_sounds_toggle_value: bool,
    keybinding_values: [input::Button; 7],
    /// Button state for flashing the keybinding button when the corresponding button is pressed.
    buttons_pressed: [bool; 7],
    keybindings_key_capture: Option<usize>,
}

fn button_name(button: input::Button) -> &'static str {
    use piston::input::{
        Button::*,
        keyboard::Key,
        mouse::MouseButton,
    };
    match button {
        Keyboard(k) => match k {
            Key::Unknown => "K_Unknown",
            Key::Backspace => "K_Backspace",
            Key::Tab => "K_Tab",
            Key::Return => "K_Return",
            Key::Escape => "K_Escape",
            Key::Space => "K_Space",
            Key::Exclaim => "K_Exclaim",
            Key::Quotedbl => "K_Quotedbl",
            Key::Hash => "K_Hash",
            Key::Dollar => "K_Dollar",
            Key::Percent => "K_Percent",
            Key::Ampersand => "K_Ampersand",
            Key::Quote => "K_Quote",
            Key::LeftParen => "K_LeftParen",
            Key::RightParen => "K_RightParen",
            Key::Asterisk => "K_Asterisk",
            Key::Plus => "K_Plus",
            Key::Comma => "K_Comma",
            Key::Minus => "K_Minus",
            Key::Period => "K_Period",
            Key::Slash => "K_Slash",
            Key::D0 => "K_D0",
            Key::D1 => "K_D1",
            Key::D2 => "K_D2",
            Key::D3 => "K_D3",
            Key::D4 => "K_D4",
            Key::D5 => "K_D5",
            Key::D6 => "K_D6",
            Key::D7 => "K_D7",
            Key::D8 => "K_D8",
            Key::D9 => "K_D9",
            Key::Colon => "K_Colon",
            Key::Semicolon => "K_Semicolon",
            Key::Less => "K_Less",
            Key::Equals => "K_Equals",
            Key::Greater => "K_Greater",
            Key::Question => "K_Question",
            Key::At => "K_At",
            Key::LeftBracket => "K_LeftBracket",
            Key::Backslash => "K_Backslash",
            Key::RightBracket => "K_RightBracket",
            Key::Caret => "K_Caret",
            Key::Underscore => "K_Underscore",
            Key::Backquote => "K_Backquote",
            Key::A => "K_A",
            Key::B => "K_B",
            Key::C => "K_C",
            Key::D => "K_D",
            Key::E => "K_E",
            Key::F => "K_F",
            Key::G => "K_G",
            Key::H => "K_H",
            Key::I => "K_I",
            Key::J => "K_J",
            Key::K => "K_K",
            Key::L => "K_L",
            Key::M => "K_M",
            Key::N => "K_N",
            Key::O => "K_O",
            Key::P => "K_P",
            Key::Q => "K_Q",
            Key::R => "K_R",
            Key::S => "K_S",
            Key::T => "K_T",
            Key::U => "K_U",
            Key::V => "K_V",
            Key::W => "K_W",
            Key::X => "K_X",
            Key::Y => "K_Y",
            Key::Z => "K_Z",
            Key::Delete => "K_Delete",
            Key::CapsLock => "K_CapsLock",
            Key::F1 => "K_F1",
            Key::F2 => "K_F2",
            Key::F3 => "K_F3",
            Key::F4 => "K_F4",
            Key::F5 => "K_F5",
            Key::F6 => "K_F6",
            Key::F7 => "K_F7",
            Key::F8 => "K_F8",
            Key::F9 => "K_F9",
            Key::F10 => "K_F10",
            Key::F11 => "K_F11",
            Key::F12 => "K_F12",
            Key::PrintScreen => "K_PrintScreen",
            Key::ScrollLock => "K_ScrollLock",
            Key::Pause => "K_Pause",
            Key::Insert => "K_Insert",
            Key::Home => "K_Home",
            Key::PageUp => "K_PageUp",
            Key::End => "K_End",
            Key::PageDown => "K_PageDown",
            Key::Right => "K_Right",
            Key::Left => "K_Left",
            Key::Down => "K_Down",
            Key::Up => "K_Up",
            Key::NumLockClear => "K_NumLockClear",
            Key::NumPadDivide => "K_NumPadDivide",
            Key::NumPadMultiply => "K_NumPadMultiply",
            Key::NumPadMinus => "K_NumPadMinus",
            Key::NumPadPlus => "K_NumPadPlus",
            Key::NumPadEnter => "K_NumPadEnter",
            Key::NumPad1 => "K_NumPad1",
            Key::NumPad2 => "K_NumPad2",
            Key::NumPad3 => "K_NumPad3",
            Key::NumPad4 => "K_NumPad4",
            Key::NumPad5 => "K_NumPad5",
            Key::NumPad6 => "K_NumPad6",
            Key::NumPad7 => "K_NumPad7",
            Key::NumPad8 => "K_NumPad8",
            Key::NumPad9 => "K_NumPad9",
            Key::NumPad0 => "K_NumPad0",
            Key::NumPadPeriod => "K_NumPadPeriod",
            Key::Application => "K_Application",
            Key::Power => "K_Power",
            Key::NumPadEquals => "K_NumPadEquals",
            Key::F13 => "K_F13",
            Key::F14 => "K_F14",
            Key::F15 => "K_F15",
            Key::F16 => "K_F16",
            Key::F17 => "K_F17",
            Key::F18 => "K_F18",
            Key::F19 => "K_F19",
            Key::F20 => "K_F20",
            Key::F21 => "K_F21",
            Key::F22 => "K_F22",
            Key::F23 => "K_F23",
            Key::F24 => "K_F24",
            Key::Execute => "K_Execute",
            Key::Help => "K_Help",
            Key::Menu => "K_Menu",
            Key::Select => "K_Select",
            Key::Stop => "K_Stop",
            Key::Again => "K_Again",
            Key::Undo => "K_Undo",
            Key::Cut => "K_Cut",
            Key::Copy => "K_Copy",
            Key::Paste => "K_Paste",
            Key::Find => "K_Find",
            Key::Mute => "K_Mute",
            Key::VolumeUp => "K_VolumeUp",
            Key::VolumeDown => "K_VolumeDown",
            Key::NumPadComma => "K_NumPadComma",
            Key::NumPadEqualsAS400 => "K_NumPadEqualsAS400",
            Key::AltErase => "K_AltErase",
            Key::Sysreq => "K_Sysreq",
            Key::Cancel => "K_Cancel",
            Key::Clear => "K_Clear",
            Key::Prior => "K_Prior",
            Key::Return2 => "K_Return2",
            Key::Separator => "K_Separator",
            Key::Out => "K_Out",
            Key::Oper => "K_Oper",
            Key::ClearAgain => "K_ClearAgain",
            Key::CrSel => "K_CrSel",
            Key::ExSel => "K_ExSel",
            Key::NumPad00 => "K_NumPad00",
            Key::NumPad000 => "K_NumPad000",
            Key::ThousandsSeparator => "K_ThousandsSeparator",
            Key::DecimalSeparator => "K_DecimalSeparator",
            Key::CurrencyUnit => "K_CurrencyUnit",
            Key::CurrencySubUnit => "K_CurrencySubUnit",
            Key::NumPadLeftParen => "K_NumPadLeftParen",
            Key::NumPadRightParen => "K_NumPadRightParen",
            Key::NumPadLeftBrace => "K_NumPadLeftBrace",
            Key::NumPadRightBrace => "K_NumPadRightBrace",
            Key::NumPadTab => "K_NumPadTab",
            Key::NumPadBackspace => "K_NumPadBackspace",
            Key::NumPadA => "K_NumPadA",
            Key::NumPadB => "K_NumPadB",
            Key::NumPadC => "K_NumPadC",
            Key::NumPadD => "K_NumPadD",
            Key::NumPadE => "K_NumPadE",
            Key::NumPadF => "K_NumPadF",
            Key::NumPadXor => "K_NumPadXor",
            Key::NumPadPower => "K_NumPadPower",
            Key::NumPadPercent => "K_NumPadPercent",
            Key::NumPadLess => "K_NumPadLess",
            Key::NumPadGreater => "K_NumPadGreater",
            Key::NumPadAmpersand => "K_NumPadAmpersand",
            Key::NumPadDblAmpersand => "K_NumPadDblAmpersand",
            Key::NumPadVerticalBar => "K_NumPadVerticalBar",
            Key::NumPadDblVerticalBar => "K_NumPadDblVerticalBar",
            Key::NumPadColon => "K_NumPadColon",
            Key::NumPadHash => "K_NumPadHash",
            Key::NumPadSpace => "K_NumPadSpace",
            Key::NumPadAt => "K_NumPadAt",
            Key::NumPadExclam => "K_NumPadExclam",
            Key::NumPadMemStore => "K_NumPadMemStore",
            Key::NumPadMemRecall => "K_NumPadMemRecall",
            Key::NumPadMemClear => "K_NumPadMemClear",
            Key::NumPadMemAdd => "K_NumPadMemAdd",
            Key::NumPadMemSubtract => "K_NumPadMemSubtract",
            Key::NumPadMemMultiply => "K_NumPadMemMultiply",
            Key::NumPadMemDivide => "K_NumPadMemDivide",
            Key::NumPadPlusMinus => "K_NumPadPlusMinus",
            Key::NumPadClear => "K_NumPadClear",
            Key::NumPadClearEntry => "K_NumPadClearEntry",
            Key::NumPadBinary => "K_NumPadBinary",
            Key::NumPadOctal => "K_NumPadOctal",
            Key::NumPadDecimal => "K_NumPadDecimal",
            Key::NumPadHexadecimal => "K_NumPadHexadecimal",
            Key::LCtrl => "K_LCtrl",
            Key::LShift => "K_LShift",
            Key::LAlt => "K_LAlt",
            Key::LGui => "K_LGui",
            Key::RCtrl => "K_RCtrl",
            Key::RShift => "K_RShift",
            Key::RAlt => "K_RAlt",
            Key::RGui => "K_RGui",
            Key::Mode => "K_Mode",
            Key::AudioNext => "K_AudioNext",
            Key::AudioPrev => "K_AudioPrev",
            Key::AudioStop => "K_AudioStop",
            Key::AudioPlay => "K_AudioPlay",
            Key::AudioMute => "K_AudioMute",
            Key::MediaSelect => "K_MediaSelect",
            Key::Www => "K_Www",
            Key::Mail => "K_Mail",
            Key::Calculator => "K_Calculator",
            Key::Computer => "K_Computer",
            Key::AcSearch => "K_AcSearch",
            Key::AcHome => "K_AcHome",
            Key::AcBack => "K_AcBack",
            Key::AcForward => "K_AcForward",
            Key::AcStop => "K_AcStop",
            Key::AcRefresh => "K_AcRefresh",
            Key::AcBookmarks => "K_AcBookmarks",
            Key::BrightnessDown => "K_BrightnessDown",
            Key::BrightnessUp => "K_BrightnessUp",
            Key::DisplaySwitch => "K_DisplaySwitch",
            Key::KbdIllumToggle => "K_KbdIllumToggle",
            Key::KbdIllumDown => "K_KbdIllumDown",
            Key::KbdIllumUp => "K_KbdIllumUp",
            Key::Eject => "K_Eject",
            Key::Sleep => "K_Sleep",
        }
        Mouse(m) => match m {
            MouseButton::Unknown => "M_Unknown",
            MouseButton::Left => "M_Left",
            MouseButton::Right => "M_Right",
            MouseButton::Middle => "M_Middle",
            MouseButton::X1 => "M_X1",
            MouseButton::X2 => "M_X2",
            MouseButton::Button6 => "M_Button6",
            MouseButton::Button7 => "M_Button7",
            MouseButton::Button8 => "M_Button8",
        }
        _ => "???",
    }
}

impl Options {
    pub(super) fn new(window: &WindowContext, config: &Config) -> Self {
        let size = window.window.size();
        let mut ui = conrod::UiBuilder::new([size.width, size.height]).build();
        ui.handle_event(conrod::event::Input::Motion(conrod::input::Motion::MouseCursor { x: window.mouse_position[0], y: window.mouse_position[1] }));
        ui.theme.font_id = Some(ui.fonts.insert(window.glyph_cache.font.clone()));
        ui.theme.shape_color = conrod::color::CHARCOAL;
        ui.theme.label_color = conrod::color::WHITE;
        ui.set_num_redraw_frames(10); // just to be safe
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
        let win_res_w_text = config.general.resolution[0].to_string();
        let win_res_h_text = config.general.resolution[1].to_string();
        let audio_buf_size_input_text = match config.general.audio_buffer_size {
            cpal::BufferSize::Fixed(n) => n.to_string(),
            cpal::BufferSize::Default => String::from("default"), // TODO
        };
        let audio_offset_input_text = config.game.offset.to_string();
        let scroll_speed_input_text = config.game.scroll_speed.to_string();
        let enable_osu_hit_sounds_toggle_value = config.game.osu_hitsound_enable;
        let keybinding_values = config.game.key_bindings;
        let buttons_pressed = [false; 7];
        let keybindings_key_capture = None;
        Self {
            ui,
            ids,
            map,
            glyph_cache,
            glyph_cache_texture,
            win_res_w_text,
            win_res_h_text,
            audio_buf_size_input_text,
            audio_offset_input_text,
            scroll_speed_input_text,
            enable_osu_hit_sounds_toggle_value,
            keybinding_values,
            buttons_pressed,
            keybindings_key_capture,
        }
    }
    pub(super) fn event(
        &mut self,
        e: piston::input::Event,
        config: &mut Config,
        _audio: &audio::Audio,
        window_context: &mut WindowContext,
    ) {
        let size = window_context.window.size();
        if let Some(e) = conrod_piston::event::convert(e.clone(), size.width, size.height) {
            self.ui.handle_event(e);
        }
        if let Some(e) = e.button_args() {
            self.keybinding_values
                .iter()
                .zip(self.buttons_pressed.iter_mut())
                .filter(|&(&k, _)| k == e.button)
                .for_each(|(_, b)| *b = match e.state {
                    input::ButtonState::Press => true,
                    input::ButtonState::Release => false,
                });
        }
        if let (Some(button), Some(key_index)) = (e.press_args(), self.keybindings_key_capture) {
            self.keybinding_values[key_index] = button;
            self.keybindings_key_capture = None;
        }
        if let Some(_) = e.update_args() {
            // Set the UI
            self.set_ui(config, window_context);
        }
        if let Some(r) = e.render_args() {
            window_context.gl.draw(r.viewport(), |c, gl| {
                if let Some(primitives) = self.ui.draw_if_changed() {
                    graphics::clear([0.0, 0.0, 0.0, 1.0], gl);
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
    fn set_ui(&mut self, config: &mut Config, window_context: &mut WindowContext) {
        let back_button;
        {
            let ui = &mut self.ui.set_widgets();

            conrod::widget::Canvas::new()
                .w(640.0)
                .border(0.0)
                .pad(50.0)
                .scroll_kids_vertically()
                .set(self.ids.main_canvas, ui);

            conrod::widget::Scrollbar::y_axis(self.ids.main_canvas)
                .auto_hide(true)
                .set(self.ids.main_scrollbar, ui);

            { // Window resolution setting
                // Invisible container around the whole setting to simplify positioning
                conrod::widget::Canvas::new()
                    .kid_area_w_of(self.ids.main_canvas)
                    .h(20.0)
                    .top_right_of(self.ids.main_canvas)
                    .border(0.0)
                    .set(self.ids.win_res_canvas, ui);

                // Text description
                conrod::widget::Text::new("Window resolution")
                    .font_size(ui.theme().font_size_small)
                    .top_left_of(self.ids.win_res_canvas)
                    .set(self.ids.win_res_text, ui);

                // height field
                let self_win_res_h_text = &mut self.win_res_h_text;
                // Make the background red if what's inside isn't a number
                let color = match self_win_res_h_text.parse::<f64>() {
                    Ok(n) if n > 0.0 => ui.theme().shape_color,
                    _ => conrod::color::RED,
                };
                conrod::widget::TextBox::new(self_win_res_h_text)
                    .font_size(ui.theme().font_size_small)
                    .w_h(50.0, 20.0)
                    .top_right_of(self.ids.win_res_canvas)
                    .color(color)
                    .border_color(conrod::color::WHITE)
                    .set(self.ids.win_res_h_input, ui)
                    .into_iter()
                    .fold(None, |a, e| if let conrod::widget::text_box::Event::Update(s) = e { Some(s) } else { a })
                    .map(|s| *self_win_res_h_text = s);

                // The "x" in between
                conrod::widget::Text::new("x")
                    .font_size(ui.theme().font_size_small)
                    .left(5.0)
                    .set(self.ids.win_res_x_text, ui);

                // width field
                let self_win_res_w_text = &mut self.win_res_w_text;
                // Make the background red if what's inside isn't a number
                let color = match self_win_res_w_text.parse::<f64>() {
                    Ok(n) if n > 0.0 => ui.theme().shape_color,
                    _ => conrod::color::RED,
                };
                conrod::widget::TextBox::new(self_win_res_w_text)
                    .font_size(ui.theme().font_size_small)
                    .w_h(50.0, 20.0)
                    .left(5.0)
                    .color(color)
                    .border_color(conrod::color::WHITE)
                    .set(self.ids.win_res_w_input, ui)
                    .into_iter()
                    .fold(None, |a, e| if let conrod::widget::text_box::Event::Update(s) = e { Some(s) } else { a })
                    .map(|s| *self_win_res_w_text = s);
            }

            { // Audio buffer size setting
                // Invisible container around the whole setting to simplify positioning
                conrod::widget::Canvas::new()
                    .kid_area_w_of(self.ids.main_canvas)
                    .h(20.0)
                    .top_right_of(self.ids.main_canvas) // align to inner right side of main canvas (inside the padding)
                    .down(20.0) // 20 pixels down from the previous widget
                    .border(0.0)
                    .set(self.ids.audio_buf_size_canvas, ui);

                // Text description
                conrod::widget::Text::new("Audio buffer size")
                    .font_size(ui.theme().font_size_small)
                    .top_left_of(self.ids.audio_buf_size_canvas)
                    .set(self.ids.audio_buf_size_text, ui);

                // Input field
                let self_audio_buf_size_input_text = &mut self.audio_buf_size_input_text;
                let color = match self_audio_buf_size_input_text.parse::<usize>() {
                    Ok(n) if n > 0 => ui.theme().shape_color,
                    _ if self_audio_buf_size_input_text == "default" => ui.theme().shape_color,
                    _ => conrod::color::RED,
                };
                conrod::widget::TextBox::new(self_audio_buf_size_input_text)
                    .font_size(ui.theme().font_size_small)
                    .w_h(60.0, 20.0)
                    .top_right_of(self.ids.audio_buf_size_canvas)
                    .color(color)
                    .border_color(conrod::color::WHITE)
                    .set(self.ids.audio_buf_size_input, ui)
                    .into_iter()
                    .fold(None, |a, e| if let conrod::widget::text_box::Event::Update(s) = e { Some(s) } else { a })
                    .map(|s| *self_audio_buf_size_input_text = s);
            }

            { // Audio offset setting
                // Invisible container around the whole setting to simplify positioning
                conrod::widget::Canvas::new()
                    .kid_area_w_of(self.ids.main_canvas)
                    .h(20.0)
                    .top_right_of(self.ids.main_canvas) // align to inner right side of main canvas (inside the padding)
                    .down(20.0) // 20 pixels down from the previous widget
                    .border(0.0)
                    .set(self.ids.audio_offset_canvas, ui);

                // Text description
                conrod::widget::Text::new("Audio offset (measured in seconds)")
                    .font_size(ui.theme().font_size_small)
                    .top_left_of(self.ids.audio_offset_canvas)
                    .set(self.ids.audio_offset_text, ui);

                // Input field
                let self_audio_offset_input_text = &mut self.audio_offset_input_text;
                let color = match self_audio_offset_input_text.parse::<f64>() {
                    Ok(n) if n.is_finite() => ui.theme().shape_color,
                    _ => conrod::color::RED,
                };
                conrod::widget::TextBox::new(self_audio_offset_input_text)
                    .font_size(ui.theme().font_size_small)
                    .w_h(50.0, 20.0)
                    .top_right_of(self.ids.audio_offset_canvas)
                    .color(color)
                    .border_color(conrod::color::WHITE)
                    .set(self.ids.audio_offset_input, ui)
                    .into_iter()
                    .fold(None, |a, e| if let conrod::widget::text_box::Event::Update(s) = e { Some(s) } else { a })
                    .map(|s| *self_audio_offset_input_text = s);
            }

            { // Scroll speed setting
                // Invisible container around the whole setting to simplify positioning
                conrod::widget::Canvas::new()
                    .kid_area_w_of(self.ids.main_canvas)
                    .h(20.0)
                    .top_right_of(self.ids.main_canvas) // align to inner right side of main canvas (inside the padding)
                    .down(20.0) // 20 pixels down from the previous widget
                    .border(0.0)
                    .set(self.ids.scroll_speed_canvas, ui);

                // Text description
                conrod::widget::Text::new("Scroll speed (measured in lane heights per second)")
                    .font_size(ui.theme().font_size_small)
                    .top_left_of(self.ids.scroll_speed_canvas)
                    .set(self.ids.scroll_speed_text, ui);

                // Input field
                let self_scroll_speed_input_text = &mut self.scroll_speed_input_text;
                let color = match self_scroll_speed_input_text.parse::<f64>() {
                    Ok(n) if n.is_finite() => ui.theme().shape_color,
                    _ => conrod::color::RED,
                };
                conrod::widget::TextBox::new(self_scroll_speed_input_text)
                    .font_size(ui.theme().font_size_small)
                    .w_h(50.0, 20.0)
                    .top_right_of(self.ids.scroll_speed_canvas)
                    .color(color)
                    .border_color(conrod::color::WHITE)
                    .set(self.ids.scroll_speed_input, ui)
                    .into_iter()
                    .fold(None, |a, e| if let conrod::widget::text_box::Event::Update(s) = e { Some(s) } else { a })
                    .map(|s| *self_scroll_speed_input_text = s);
            }

            { // Enable osu hitsounds setting
                // Invisible container around the whole setting to simplify positioning
                conrod::widget::Canvas::new()
                    .kid_area_w_of(self.ids.main_canvas)
                    .h(20.0)
                    .top_right_of(self.ids.main_canvas) // align to inner right side of main canvas (inside the padding)
                    .down(20.0) // 20 pixels down from the previous widget
                    .border(0.0)
                    .set(self.ids.enable_osu_hit_sounds_canvas, ui);

                // Text description
                conrod::widget::Text::new("Enable osu hitsounds")
                    .font_size(ui.theme().font_size_small)
                    .top_left_of(self.ids.enable_osu_hit_sounds_canvas)
                    .set(self.ids.enable_osu_hit_sounds_text, ui);

                // Input field
                let self_enable_osu_hit_sounds_toggle_value = &mut self.enable_osu_hit_sounds_toggle_value;
                conrod::widget::Toggle::new(*self_enable_osu_hit_sounds_toggle_value)
                    .w_h(20.0, 20.0)
                    .top_right_of(self.ids.enable_osu_hit_sounds_canvas)
                    .border_color(conrod::color::WHITE)
                    .color(conrod::color::WHITE)
                    .set(self.ids.enable_osu_hit_sounds_toggle, ui)
                    .last()
                    .map(|v| *self_enable_osu_hit_sounds_toggle_value = v);
            }

            { // Keybindings
                // Invisible container around the whole setting to simplify positioning
                conrod::widget::Canvas::new()
                    .kid_area_w_of(self.ids.main_canvas)
                    .h(50.0)
                    .top_right_of(self.ids.main_canvas) // align to inner right side of main canvas (inside the padding)
                    .down(20.0) // 20 pixels down from the previous widget
                    .border(0.0)
                    .set(self.ids.keybindings_canvas, ui);

                // Text description
                conrod::widget::Text::new("Keybindings")
                    .font_size(ui.theme().font_size_small)
                    .mid_top_of(self.ids.keybindings_canvas)
                    .set(self.ids.keybindings_text, ui);

                // Container containing the button to click to bind controls
                conrod::widget::Canvas::new()
                    .kid_area_w_of(self.ids.keybindings_canvas)
                    .h(20.0)
                    .mid_bottom_of(self.ids.keybindings_canvas)
                    .border(0.0)
                    .flow_right(&[
                        (self.ids.key0_canvas, conrod::widget::Canvas::new().h(20.0).pad_left(1.0).pad_right(1.0)),
                        (self.ids.key1_canvas, conrod::widget::Canvas::new().h(20.0).pad_left(1.0).pad_right(1.0)),
                        (self.ids.key2_canvas, conrod::widget::Canvas::new().h(20.0).pad_left(1.0).pad_right(1.0)),
                        (self.ids.key3_canvas, conrod::widget::Canvas::new().h(20.0).pad_left(1.0).pad_right(1.0)),
                        (self.ids.key4_canvas, conrod::widget::Canvas::new().h(20.0).pad_left(1.0).pad_right(1.0)),
                        (self.ids.key5_canvas, conrod::widget::Canvas::new().h(20.0).pad_left(1.0).pad_right(1.0)),
                        (self.ids.key6_canvas, conrod::widget::Canvas::new().h(20.0).pad_left(1.0).pad_right(1.0)),
                    ])
                    .set(self.ids.keybindings_buttons_canvas, ui);

                let canvas_ids = [
                    self.ids.key0_canvas,
                    self.ids.key1_canvas,
                    self.ids.key2_canvas,
                    self.ids.key3_canvas,
                    self.ids.key4_canvas,
                    self.ids.key5_canvas,
                    self.ids.key6_canvas,
                ];
                let button_ids = [
                    self.ids.key0_button,
                    self.ids.key1_button,
                    self.ids.key2_button,
                    self.ids.key3_button,
                    self.ids.key4_button,
                    self.ids.key5_button,
                    self.ids.key6_button,
                ];
                for (i, (((&canvas_id, &button_id), &button), &key_pressed)) in canvas_ids
                    .iter()
                        .zip(button_ids.iter())
                        .zip(self.keybinding_values.iter())
                        .zip(self.buttons_pressed.iter())
                        .enumerate() {
                    let mut button = conrod::widget::Button::new()
                        .top_left_of(canvas_id)
                        .kid_area_wh_of(canvas_id)
                        .label(button_name(button))
                        .label_font_size(8);
                    if Some(i) == self.keybindings_key_capture {
                        // white border if waiting for a button press to bind to
                        button = button.border(1.0).border_color(conrod::color::WHITE);
                    } else if key_pressed {
                        // red border if currently bound button is being pressed
                        button = button.border(2.0).border_color(conrod::color::RED);
                    } else {
                        // otherwise noborder
                        button = button.border(0.0);
                    }
                    if button.set(button_id, ui).was_clicked() {
                        self.keybindings_key_capture = Some(i);
                    }
                }
            }

            // back button
            back_button = conrod::widget::Button::new()
                .top_left_of(ui.window)
                .w_h(35.0, 25.0)
                .label("back")
                .small_font(&ui)
                .set(self.ids.back_button, ui);
        }

        if back_button.was_clicked() {
            self.update_config(config);
            let config_path = config::config_path();
            match config::write_config_to_path(config.clone(), &config_path) {
                Err(e) => remani_warn!("Error writing config to {}: {}", config_path.display(), e),
                Ok(()) => (),
            }
            window_context.change_scene(MainMenu::new());
        }
    }
    fn update_config(&self, config: &mut Config) {
        match self.win_res_w_text
            .parse()
            .and_then(|w| self.win_res_h_text.parse().map(|h| [w, h]))
        {
            Ok(res) => config.general.resolution = res,
            Err(_) => remani_warn!("Failed to parse resolution, ignoring..."),
        }
        // TODO trigger audio device reload
        match self.audio_buf_size_input_text.parse() {
            Ok(n) if n > 0 => config.general.audio_buffer_size = cpal::BufferSize::Fixed(n),
            _ if self.audio_buf_size_input_text == "default" => config.general.audio_buffer_size = cpal::BufferSize::Default,

            Ok(_) => remani_warn!("Invalid audio buffer size, ignoring..."),
            _ => remani_warn!("Failed to parse audio buffer size, ignoring..."),
        }

        match self.audio_offset_input_text.parse() {
            Ok(n) => config.game.offset = n,
            Err(_) => remani_warn!("Failed to parse audio offset, ignoring..."),
        }

        match self.scroll_speed_input_text.parse() {
            Ok(n) => config.game.scroll_speed = n,
            Err(_) => remani_warn!("Failed to parse scroll speed, ignoring..."),
        }

        config.game.osu_hitsound_enable = self.enable_osu_hit_sounds_toggle_value;
        config.game.key_bindings = self.keybinding_values;
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
