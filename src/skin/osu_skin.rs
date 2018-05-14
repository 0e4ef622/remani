//! Osu skin directory parser module

use image;

use opengl_graphics::Texture;
use opengl_graphics::GlGraphics;
use graphics::image::Image;
use graphics::draw_state::DrawState;
use graphics::math;
use graphics::Transformed;
use texture::TextureSettings;
use texture::ImageSize;
use std::ops::Deref;

use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::rc::Rc;
use std::collections::HashMap;
use std::path;
use std::error;
use std::fmt;
use std::time;

use skin::{ Skin, ParseError };
use judgement::Judgement;

#[derive(Copy, Clone, Debug)]
enum NoteBodyStyle {
    Stretch,
    CascadeFromTop,
    CascadeFromBottom,
}

/// Holds skin data, such as note images and what not.
struct OsuSkinTextures {
    miss: Rc<Vec<Rc<Texture>>>,
    hit50: Rc<Vec<Rc<Texture>>>,
    hit100: Rc<Vec<Rc<Texture>>>,
    hit200: Rc<Vec<Rc<Texture>>>,
    hit300: Rc<Vec<Rc<Texture>>>,
    hit300g: Rc<Vec<Rc<Texture>>>,

    /// The images virtual keys under the judgement line.
    keys: [Rc<Texture>; 7],

    /// The images of the virtual keys under the judgement line when the
    /// corresponding key on the keyboard is pressed.
    keys_d: [Rc<Texture>; 7],

    /// The notes' images.
    notes: [Rc<Vec<Rc<Texture>>>; 7],

    /// The long notes' ends' images.
    long_notes_head: [Rc<Vec<Rc<Texture>>>; 7],

    /// The long notes' bodies' images.
    long_notes_body: [Rc<Vec<Rc<Texture>>>; 7],

    /// The long notes' tails' images.
    long_notes_tail: [Option<Rc<Vec<Rc<Texture>>>>; 7],

    /// The stage light animation images
    stage_light: Rc<Vec<Rc<Texture>>>,

    /// The stage components.
    stage_hint: Rc<Vec<Rc<Texture>>>,
    stage_left: Rc<Texture>,
    stage_right: Rc<Texture>,
    stage_bottom: Option<Rc<Vec<Rc<Texture>>>>,
}

struct OsuSkinConfig {
    /// Various information related to how to draw components.
    column_start: u16,
    column_width: [u16; 7],
    column_spacing: [u16; 6],
    column_line_width: [u16; 8],
    hit_position: u16,
    score_position: u16,
    width_for_note_height_scale: f64,
    note_body_style: [NoteBodyStyle; 7],

    colour_light: [[u8; 3]; 7],

    // TODO
    // lighting_n_width: [u16; 7],
    // lighting_l_width: [u16; 7],
    // combo_position: u16,
    // judgement_line: bool,

    // low priority
    // special_style: SpecialStyle,
    // keys_under_notes: bool,
}

struct OsuAnimStates {
    keys_last_down_time: [Option<time::Instant>; 7],
}

struct OsuSkin {
    textures: OsuSkinTextures,
    config: OsuSkinConfig,
    anim_states: OsuAnimStates,

    /// judgement, time of first frame
    judgement: Option<(Judgement, time::Instant)>,
}

impl Skin for OsuSkin {
    fn draw_play_scene(&mut self,
                       draw_state: &DrawState,
                       transform: math::Matrix2d,
                       gl: &mut GlGraphics,
                       stage_height: f64,
                       keys_down: &[bool],
                       // column index, start pos, end pos
                       notes: &[(usize, f64, Option<f64>)]) {

        self.draw_track(draw_state, transform, gl, stage_height);
        for &(column, pos, end_pos) in notes {
            if let Some(end_p) = end_pos {
                self.draw_long_note(draw_state, transform, gl, stage_height, pos, end_p, column);
            } else {
                self.draw_note(draw_state, transform, gl, stage_height, pos, column);
            }
        }
        self.draw_keys(draw_state, transform, gl, stage_height, keys_down);

        // Draw judgement
        if let Some((judgement, time)) = self.judgement {
            let elapsed = time.elapsed();

            if elapsed <= time::Duration::from_millis(200) {

                // the "burst" animation
                let scale = if elapsed <= time::Duration::from_millis(50) {
                    1.5 - elapsed.subsec_nanos() as f64 / 50_000_000.0 / 2.0
                } else if elapsed <= time::Duration::from_millis(160) {
                    1.0
                } else {
                    1.0 - (elapsed.subsec_nanos() - 160_000_000) as f64 / 150_000_000.0
                };
                match judgement {
                    Judgement::Miss => self.draw_miss(draw_state, transform, gl, stage_height),
                    Judgement::Bad => (), // TODO
                    Judgement::Good => (),
                    Judgement::Perfect => self.draw_perfect(draw_state, transform, scale, gl, stage_height, elapsed),
                };
            } else {
                self.judgement = None;
            }
        }
    }

    fn draw_judgement(&mut self, _column: usize, judgement: Judgement) {
        self.judgement = Some((judgement, time::Instant::now()));
    }

    fn key_down(&mut self, column: usize) {
        self.anim_states.keys_last_down_time[column] = None;
    }

    fn key_up(&mut self, column: usize) {
        self.anim_states.keys_last_down_time[column] = Some(time::Instant::now());
    }
}

impl OsuSkin {
    // TODO render animations
    fn draw_note(&self, draw_state: &DrawState, transform: math::Matrix2d, gl: &mut GlGraphics, stage_h: f64, pos: f64, column_index: usize) {

        let scale = stage_h / 480.0;
        let hit_p = self.config.hit_position as f64 * scale;

        let note_w = self.config.column_width[column_index] as f64 * scale;
        let note_h = self.config.width_for_note_height_scale * scale;
        let note_x = scale * (self.config.column_start as f64 +
                              self.config.column_width[0..column_index].iter().sum::<u16>() as f64 +
                              self.config.column_spacing[0..column_index].iter().sum::<u16>() as f64);

        let note_y = hit_p * (1.0 - pos) - note_h;

        let note = self.textures.notes[column_index][0].deref();
        let note_img = Image::new().rect([note_x, note_y, note_w, note_h]);
        note_img.draw(note, draw_state, transform, gl);
    }
    fn draw_long_note(&self, draw_state: &DrawState, transform: math::Matrix2d, gl: &mut GlGraphics,
                      stage_h: f64, pos: f64, end_pos: f64, column_index: usize) {

        let scale = stage_h / 480.0;
        let scale2 = stage_h / 768.0; // long note body height when cascading is scaled with this
        let hit_p = self.config.hit_position as f64 * scale;

        let note_w = self.config.column_width[column_index] as f64 * scale;
        let note_x = scale * (self.config.column_start as f64 +
                              self.config.column_width[0..column_index].iter().sum::<u16>() as f64 +
                              self.config.column_spacing[0..column_index].iter().sum::<u16>() as f64);
        let real_bottom_y = hit_p * (1.0 - pos);
        let bottom_y = if pos < 0.0 { hit_p } else { real_bottom_y };
        let top_y = hit_p * (1.0 - end_pos);

        let note_head = self.textures.long_notes_head[column_index][0].deref();
        let note_tail = self.textures.long_notes_tail[column_index].as_ref().map(|v| v[0].deref());
        let note_body = self.textures.long_notes_body[column_index][0].deref();

        let note_body_h = note_body.get_height() as f64 * scale2;
        let note_end_h = self.config.width_for_note_height_scale * scale;
        let note_head_y = bottom_y - note_end_h;
        let note_tail_y = top_y - note_end_h;

        let note_head_img = Image::new().rect([note_x, note_head_y, note_w, note_end_h]);
        let note_tail_img = Image::new().rect([note_x, note_tail_y, note_w, note_end_h]);

        match self.config.note_body_style[column_index] {
            NoteBodyStyle::Stretch => {
                let note_body_img = Image::new()
                    .src_rect([0.0, 0.0, note_body.get_width() as f64, ((bottom_y - top_y)/(real_bottom_y - top_y)*(note_body.get_height() as f64))])
                    .rect([note_x, top_y - note_end_h/2.0, note_w, bottom_y - top_y]);
                note_body_img.draw(note_body, draw_state, transform, gl);
            },
            NoteBodyStyle::CascadeFromTop => {
                let mut rect = [note_x, top_y - note_end_h/2.0, note_w, note_body_h];
                let mut i = 0.0;
                let mut note_body_img = Image::new();
                while i < bottom_y - top_y - note_body_h {
                    note_body_img = note_body_img.rect(rect);
                    note_body_img.draw(note_body, draw_state, transform, gl);
                    rect[1] += note_body_h;
                    i += note_body_h;
                }
                let mut mod_rect = rect.clone();
                mod_rect[3] = bottom_y - top_y - i;
                let src_rect = [0.0, 0.0, note_body.get_width() as f64, mod_rect[3]];
                note_body_img = note_body_img.src_rect(src_rect).rect(mod_rect);
                note_body_img.draw(note_body, draw_state, transform, gl);
            },
            NoteBodyStyle::CascadeFromBottom => {
                let mut rect = [note_x, top_y - note_end_h/2.0, note_w, note_body_h];
                let mut i = 0.0;
                let mut note_body_img = Image::new();

                let offset = (real_bottom_y - top_y) % note_body_h;

                let mut mod_rect = rect.clone();
                mod_rect[3] = offset;
                let src_rect = [0.0, offset / scale2, note_body.get_width() as f64, -(mod_rect[3] / scale2)];
                note_body_img = note_body_img.src_rect(src_rect).rect(mod_rect);
                note_body_img.draw(note_body, draw_state, transform, gl);

                note_body_img = Image::new();

                rect[1] += offset;
                i = offset;

                let upside_down_rect = [0.0, note_body.get_height() as f64, note_body.get_width() as f64, -(note_body.get_height() as f64)];

                while i < bottom_y - top_y - note_body_h {
                    note_body_img = note_body_img.src_rect(upside_down_rect).rect(rect);
                    note_body_img.draw(note_body, draw_state, transform, gl);
                    rect[1] += note_body_h;
                    i += note_body_h;
                }

                mod_rect = rect.clone();
                mod_rect[3] = bottom_y - top_y - i;
                let src_rect = [0.0, note_body.get_height() as f64, note_body.get_width() as f64, -mod_rect[3]];
                note_body_img = note_body_img.src_rect(src_rect).rect(mod_rect);
                note_body_img.draw(note_body, draw_state, transform, gl);

                let note_body_img = Image::new().rect([note_x, top_y - note_end_h/2.0, note_w, bottom_y - top_y]);
                note_body_img.draw(note_body, draw_state, transform, gl);
            },
            NoteBodyStyle::CascadeFromTop => {
                let mut rect = [note_x, top_y - note_end_h/2.0, note_w, note_body.get_height() as f64];
                let mut i = 0.0;
                while i < bottom_y - top_y {
                    let mut note_body_img = Image::new();
                    if i + note_body.get_height() as f64 >= bottom_y - top_y {
                        let mut mod_rect = rect.clone();
                        mod_rect[3] = bottom_y - top_y - i;
                        let src_rect = [0.0, 0.0, note_body.get_width() as f64, mod_rect[3]];
                        note_body_img = note_body_img.src_rect(src_rect).rect(mod_rect);
                        note_body_img.draw(note_body, draw_state, transform, gl);
                    } else {
                        note_body_img = note_body_img.rect(rect);
                        note_body_img.draw(note_body, draw_state, transform, gl);
                    }
                    rect[1] += note_body.get_height() as f64;
                    i += note_body.get_height() as f64;
                }
            },
            NoteBodyStyle::CascadeFromBottom => {
                let mut rect = [note_x, bottom_y - note_end_h*2.0 - (note_body.get_height() as f64) / 2.0, note_w, note_body.get_height() as f64];
                let mut i = bottom_y - top_y;
                while i + note_body.get_height() as f64 > 0.0 {
                    let mut note_body_img = Image::new();
                    if i <= 0.0 {
                        let mut mod_rect = rect.clone();
                        mod_rect[1] = top_y - note_end_h/2.0;
                        mod_rect[3] = note_body.get_height() as f64 + i;
                        let src_rect = [0.0, -i, note_body.get_width() as f64, note_body.get_height() as f64 + i];
                        note_body_img = note_body_img.src_rect(src_rect).rect(mod_rect);
                        note_body_img.draw(note_body, draw_state, transform, gl);
                    } else {
                        note_body_img = note_body_img.rect(rect);
                        note_body_img.draw(note_body, draw_state, transform, gl);
                    }
                    rect[1] -= note_body.get_height() as f64;
                    i -= note_body.get_height() as f64;
                }
            },
        }

        if pos >= 0.0 {
            note_head_img.draw(note_head, draw_state, transform, gl);
        }

        note_head_img.draw(note_head, draw_state, transform, gl);

        if let Some(note_tail) = note_tail {
            note_tail_img.draw(note_tail, draw_state, transform, gl);
        } else {
            note_tail_img.src_rect([0.0, note_head.get_height() as f64,
                                    note_head.get_width() as f64, -(note_head.get_height() as f64)])
                         .draw(note_head, draw_state, transform, gl);
        }
    }

    fn draw_track(&self, draw_state: &DrawState, transform: math::Matrix2d, gl: &mut GlGraphics, stage_h: f64) {

        let scale = stage_h / 480.0;

        // Apparently some things are based on a height of 480, and other things are based on a
        // height of 768. .-.
        let scale2 = stage_h / 768.0;

        let column_width_sum = (self.config.column_width.iter().sum::<u16>() as f64 + self.config.column_spacing.iter().sum::<u16>() as f64) * scale;
        let column_start = self.config.column_start as f64 * scale;
        let stage_hint_height = self.textures.stage_hint[0].get_height() as f64 * scale;
        let stage_l_width = self.textures.stage_left.get_width() as f64 * scale2;
        let stage_r_width = self.textures.stage_right.get_width() as f64 * scale2;

        let stage_l_img = Image::new().rect([column_start - stage_l_width , 0.0, stage_l_width, stage_h]);
        let stage_r_img = Image::new().rect([column_start + column_width_sum, 0.0, stage_r_width, stage_h]);
        let stage_hint_img = Image::new().rect([column_start, self.config.hit_position as f64 * scale - stage_hint_height / 2.0, column_width_sum, stage_hint_height]);

        stage_hint_img.draw(self.textures.stage_hint[0].deref(), draw_state, transform, gl);
        stage_l_img.draw(self.textures.stage_left.deref(), draw_state, transform, gl);
        stage_r_img.draw(self.textures.stage_right.deref(), draw_state, transform, gl);

        if let Some(ref v) = self.textures.stage_bottom {
            let stage_bottom = &v[0];
            let stage_b_width = stage_bottom.get_width() as f64 * scale;
            let stage_b_height = stage_bottom.get_height() as f64 * scale;
            let stage_b_img = Image::new().rect([column_start + column_width_sum / 2.0 - stage_b_width / 2.0, stage_h - stage_b_height, stage_b_width, stage_b_height]);
            stage_b_img.draw(stage_bottom.deref(), draw_state, transform, gl);
        }
    }

    fn draw_keys(&self, draw_state: &DrawState, transform: math::Matrix2d, gl: &mut GlGraphics, stage_h: f64, pressed: &[bool]) {

        let scale = stage_h / 480.0;
        let scale2 = stage_h / 768.0;

        for (i, key_pressed) in pressed.iter().enumerate() {
            let key_texture: &Texture = if *key_pressed { self.textures.keys_d[i].as_ref() } else { self.textures.keys[i].as_ref() };
            let key_width = self.config.column_width[i] as f64 * scale;
            let key_height = key_texture.get_height() as f64 * scale2;
            let key_x = scale * (self.config.column_start as f64 +
                                 self.config.column_width[0..i].iter().sum::<u16>() as f64 +
                                 self.config.column_spacing[0..i].iter().sum::<u16>() as f64);
            let key_y = stage_h - key_height;
            let key_img = Image::new().rect([key_x, key_y, key_width, key_height]);
            key_img.draw(key_texture, draw_state, transform, gl);

            let mut color = [self.config.colour_light[i][0] as f32 / 255.0,
                         self.config.colour_light[i][1] as f32 / 255.0,
                         self.config.colour_light[i][2] as f32 / 255.0, 1.0];
            let sl_size = self.textures.stage_light.len();
            let stage_light_height = self.textures.stage_light[sl_size-1].get_height() as f64 * scale2;

            if self.anim_states.keys_last_down_time[i] != None {
                let current_time = time::Instant::now();
                let elapsed_time = current_time - self.anim_states.keys_last_down_time[i].unwrap();
                let elapsed_time_secs = elapsed_time.as_secs() as f64 + elapsed_time.subsec_nanos() as f64 / 1_000_000_000.0;
                let fframe: f32 = elapsed_time_secs as f32 * 30.0;
                let frame = fframe as usize;
                /* if frame < self.textures.stage_light.len() {
                    let color = [self.config.colour_light[i][0] as f32 / 255.0,
                                 self.config.colour_light[i][1] as f32 / 255.0,
                                 self.config.colour_light[i][2] as f32 / 255.0, 1.0];
                    let stage_light_height = self.textures.stage_light[frame].get_height() as f64 * scale2;
                    let stage_light_img = Image::new().rect([key_x, key_y - stage_light_height, key_width, stage_light_height]).color(color);
                    stage_light_img.draw(self.textures.stage_light[frame].as_ref(), draw_state, transform, gl);
                } else */
                if frame < 3 {
                    color[3] -= fframe/3.0;
                    let stage_light_img = Image::new().rect([key_x, key_y - stage_light_height, key_width, stage_light_height]).color(color);
                    stage_light_img.draw(self.textures.stage_light[sl_size-1].as_ref(), draw_state, transform, gl);
                }
            } else if *key_pressed {
                let stage_light_img = Image::new().rect([key_x, key_y - stage_light_height, key_width, stage_light_height]).color(color);
                stage_light_img.draw(self.textures.stage_light[0].as_ref(), draw_state, transform, gl);
            }
        }
    }

    fn draw_perfect(&self, draw_state: &DrawState, transform: math::Matrix2d, size_scale: f64, gl: &mut GlGraphics, stage_h: f64, elapsed_time: time::Duration) {
        let elapsed = elapsed_time.as_secs() as f64 + elapsed_time.subsec_nanos() as f64 / 1_000_000_000.0;
        //let frame = ((elapsed % (self.textures.hit300g.len() as f64 / 60.0)) / 60.0) as usize;
        let frame = (elapsed * 30.0) as usize % self.textures.hit300g.len();

        let tx = self.textures.hit300g[frame].deref();

        let scale = stage_h / 480.0;
        let scale2 = stage_h / 768.0;
        let score_p = self.config.hit_position as f64 * scale;
        let stage_width = (self.config.column_width.iter().sum::<u16>() as f64 + self.config.column_spacing.iter().sum::<u16>() as f64) * scale;
        let column_start = self.config.column_start as f64 * scale;

        let tx_w = tx.get_width() as f64 * scale2 / 1.5 * size_scale;
        let tx_h = tx.get_height() as f64 * scale2 / 1.5 * size_scale;
        let tx_x = stage_width / 2.0 - tx_w / 2.0 + column_start;
        let tx_y = self.config.score_position as f64 * scale - tx_h / 2.0;

        let img = Image::new().rect([tx_x, tx_y, tx_w, tx_h]);
        img.draw(tx, draw_state, transform, gl);
    }

    fn draw_miss(&self, draw_state: &DrawState, transform: math::Matrix2d, gl: &mut GlGraphics, stage_h: f64) {
        let tx = self.textures.miss[0].deref();

        let scale = stage_h / 480.0;
        let scale2 = stage_h / 768.0;
        let score_p = self.config.hit_position as f64 * scale;
        let stage_width = (self.config.column_width.iter().sum::<u16>() as f64 + self.config.column_spacing.iter().sum::<u16>() as f64) * scale;
        let column_start = self.config.column_start as f64 * scale;

        let tx_w = tx.get_width() as f64 * scale2;
        let tx_h = tx.get_height() as f64 * scale2;
        let tx_x = stage_width / 2.0 - tx_w / 2.0 + column_start;
        let tx_y = self.config.score_position as f64 * scale - tx_h / 2.0;

        let img = Image::new().rect([tx_x, tx_y, tx_w, tx_h]);
        img.draw(tx, draw_state, transform, gl);
    }
}

#[derive(Debug)]
enum OsuSkinParseError {
    NoDefaultTexture(String),
}

impl fmt::Display for OsuSkinParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OsuSkinParseError::NoDefaultTexture(ref s) => write!(f, "No default texture found for {}", s),
        }
    }
}

impl From<OsuSkinParseError> for ParseError {
    fn from(e: OsuSkinParseError) -> ParseError {
        ParseError::Parse(String::from("Error reading osu skin"), Some(Box::new(e)))
    }
}

impl error::Error for OsuSkinParseError {
    fn description(&self) -> &str {
        match *self {
            OsuSkinParseError::NoDefaultTexture(_) => "No default texture found",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        Some(self)
    }
}

// Work around https://github.com/PistonDevelopers/opengl_graphics/issues/264
// Performs a reverse sRGB transformation
fn image_reverse_srgb(mut img: image::RgbaImage) -> image::RgbaImage {
    use std::u8;

    // We can't use graphics::color::gamma_srgb_to_linear(color) because it doesn't
    // perform the transformation on the alpha channel, which we want
    img.pixels_mut().for_each(|pixel| {

            pixel.data.iter_mut().for_each(|c| {
                const U8_MAX: f32 = u8::MAX as f32;

                let mut v = *c as f32 / U8_MAX;

                if v <= 0.04045 { v = v / 12.92 }
                else { v = ((v + 0.055) / 1.055).powf(2.4) }

                *c = (v * U8_MAX).round() as u8;

            });
        });
    img
}

fn texture_from_path<T: AsRef<path::Path>>(path: T, texture_settings: &TextureSettings) -> Result<Texture, ParseError> {
    let image = match image::open(&path) {
        Ok(t) => t,
        Err(e) => return Err(ParseError::ImageError(path.as_ref().to_string_lossy().into_owned(), e)),
    };
    Ok(Texture::from_image(&image_reverse_srgb(image.to_rgba()), texture_settings))
}

/// Load an animatable skin element's textures
fn load_texture_anim(cache: &mut HashMap<String, Rc<Vec<Rc<Texture>>>>,
                dir: &path::Path,
                default_dir: &path::Path,
                names: &(&'static str, String),
                texture_settings: &TextureSettings) -> Result<Rc<Vec<Rc<Texture>>>, ParseError> {

    let mut textures = Vec::new();
    let mut path;

    macro_rules! repetitive_code {
        ($(($dir:ident, $name:expr)),*) => {$(

            if let Some(texture) = cache.get(&$name) {
                return Ok(Rc::clone(texture));
            }

            path = $dir.join($name + ".png");
            if path.exists() {
                // help
                let texture = Rc::new(texture_from_path(&path, texture_settings)?);
                let anim = Rc::new(vec![texture]);
                cache.insert($name, Rc::clone(&anim));
                return Ok(anim);
            }

            path = $dir.join($name + "-0.png");
            if path.exists() {
                textures.push(Rc::new(texture_from_path(&path, texture_settings)?));
                let mut n = 1;
                loop {
                    path = $dir.join(format!("{}-{}.png", $name, n));
                    if !path.exists() { break; }
                    textures.push(Rc::new(texture_from_path(&path, texture_settings)?));
                    n += 1;
                }
                let anim = Rc::new(textures);
                cache.insert($name, Rc::clone(&anim));
                return Ok(anim);
            }
        )*}
    }

    repetitive_code!((dir, names.1.clone()), (default_dir, names.0.to_owned()));

    Err(OsuSkinParseError::NoDefaultTexture(String::from(names.0)).into())
}

/// Load a skin element's texture
fn load_texture(cache: &mut HashMap<String, Rc<Vec<Rc<Texture>>>>,
                dir: &path::Path,
                default_dir: &path::Path,
                names: &(&'static str, String),
                texture_settings: &TextureSettings) -> Result<Rc<Texture>, ParseError> {

    macro_rules! repetitive_code {
        ($(($dir:ident, $name:expr)),*) => {$(

            if let Some(texture) = cache.get(&$name) {
                return Ok(Rc::clone(&texture[0]));
            }

            let path = $dir.join($name + ".png");
            if path.exists() {
                let texture = texture_from_path(path, texture_settings)?;
                let rc = Rc::new(texture);
                cache.insert($name, Rc::new(vec![Rc::clone(&rc)]));
                return Ok(rc);
            }
        )*}
    }

    repetitive_code!((dir, names.1.clone()), (default_dir, names.0.to_owned()));

    Err(OsuSkinParseError::NoDefaultTexture(String::from(names.0)).into())
}

pub fn from_path(dir: &path::Path, default_dir: &path::Path) -> Result<Box<Skin>, ParseError> {
    let config_path = dir.join(path::Path::new("skin.ini"));

    let texture_settings = TextureSettings::new();

    macro_rules! double {
        ($e:expr) => (($e, String::from($e)))
    }

    // put things into the 1213121 pattern
    macro_rules! pat {
        ($a:expr, $b:expr, $c:expr) => [[$a, $b, $a, $c, $a, $b, $a]]
    }

    // (default image name, skin image name)
    // the skin filename might get changed by the skin.ini, which is parsed later
    let mut miss_name = double!("mania-hit0");
    let mut hit50_name = double!("mania-hit50");
    let mut hit100_name = double!("mania-hit100");
    let mut hit200_name = double!("mania-hit200");
    let mut hit300_name = double!("mania-hit300");
    let mut hit300g_name = double!("mania-hit300g");
    let mut stage_light_name = double!("mania-stage-light");

    let mut keys_name = pat![double!("mania-key1"),
                             double!("mania-key2"),
                             double!("mania-keyS")];

    let mut keys_d_name = pat![double!("mania-key1D"),
                               double!("mania-key2D"),
                               double!("mania-keySD")];

    let mut notes_name = pat![double!("mania-note1"),
                              double!("mania-note2"),
                              double!("mania-noteS")];

    // lns is plural of ln (long note)
    let mut lns_head_name = pat![double!("mania-note1H"),
                                 double!("mania-note2H"),
                                 double!("mania-noteSH")];

    let mut lns_body_name = pat![double!("mania-note1L"),
                                 double!("mania-note2L"),
                                 double!("mania-noteSL")];

    let mut lns_tail_name = pat![double!("mania-note1T"),
                                 double!("mania-note2T"),
                                 double!("mania-noteST")];

    let mut stage_hint_name = double!("mania-stage-hint");
    let mut stage_left_name = double!("mania-stage-left");
    let mut stage_right_name = double!("mania-stage-right");
    let mut stage_bottom_name = double!("mania-stage-bottom");

    // default values
    let mut column_start = 136;
    let mut column_width = [30; 7];
    let mut column_line_width = [2; 8];
    let mut column_spacing = [0; 6];
    let mut colour_light = [[255, 255, 255]; 7];
    let mut hit_position = 402;
    let mut score_position = 240; // idk TODO
    let mut note_body_style = [NoteBodyStyle::CascadeFromTop; 7];

    // parse skin.ini
    if config_path.exists() {
        let config_file = File::open(config_path).unwrap();
        let config_reader = BufReader::new(&config_file);
        let mut section = String::from("General");
        let mut keys: u8 = 0;
        for l in config_reader.lines() {
            let line = l.unwrap().to_string().clone().to_owned().trim().to_owned();
            if line.starts_with("[") && line.ends_with("]") {
                section = line.clone();
                section = section[1..section.len()-1].to_string();
                continue;
            }
            if line.starts_with("//") || line == "" {
                continue;
            }

            let mut line_parts = line.splitn(2, ":");

            let key = if let Some(k) = line_parts.next() { k.trim() } else { continue; };
            let value = if let Some(v) = line_parts.next() { v.trim() } else { continue; };
            match key {
                "Keys" => keys = value.parse().unwrap(),
                _ => {
                    if keys == 7 {
                        // fancy macros
                        macro_rules! enumerate_match {
                            ($key:ident, $($prefix:expr, $suffix:expr => $varname:ident = $value:expr, ($baseidx:expr, [ $($idx:expr)* ]),)*) => {
                                match $key {
                                    $($(
                                    concat!(concat!($prefix, stringify!($idx)), $suffix) => $varname[$idx - $baseidx].1 = $value,
                                    )*)*
                                    _ => (),
                                }
                            }
                        }

                        // for values that look like
                        // 42,10,5,1337,4,8,2
                        macro_rules! csv {
                            ($default:expr; $count:expr) => {{
                                let mut a = $default;
                                for (i, v) in value.split(",").enumerate().take($count) {
                                    a[i] = v.parse().unwrap();
                                }
                                a
                            }}
                        }
                        match key {
                            "ColumnStart" => column_start = value.parse().unwrap(),
                            "HitPosition" => hit_position = value.parse().unwrap(),
                            "ScorePosition" => score_position = value.parse().unwrap(),
                            "ColumnWidth" => column_width = csv![column_width; 7],
                            "ColumnLineWidth" => column_line_width = csv![column_line_width; 8],
                            "ColumnSpacing" => column_spacing = csv![column_spacing; 6],
                            "NoteBodyStyle" => for (i, v) in value.split(",").enumerate().take(7) {
                                note_body_style[i] = match v {
                                    "0" => NoteBodyStyle::Stretch,
                                    "1" => NoteBodyStyle::CascadeFromTop,
                                    "2" => NoteBodyStyle::CascadeFromBottom,
                                    _ => continue,
                                }
                            },
                            "Hit0" => miss_name.1 = value.to_owned(),
                            "Hit50" => hit50_name.1 = value.to_owned(),
                            "Hit100" => hit100_name.1 = value.to_owned(),
                            "Hit200" => hit200_name.1 = value.to_owned(),
                            "Hit300" => hit300_name.1 = value.to_owned(),
                            "Hit300g" => hit300g_name.1 = value.to_owned(),
                            "StageHint" => stage_hint_name.1 = value.to_owned(),
                            "StageLeft" => stage_left_name.1 = value.to_owned(),
                            "StageRight" => stage_right_name.1 = value.to_owned(),
                            "StageBottom" => stage_bottom_name.1 = value.to_owned(),
                            "StageLight" => stage_light_name.1 = value.to_owned(),

                            "ColourLight1" => colour_light[0] = csv![colour_light[0]; 3],
                            "ColourLight2" => colour_light[1] = csv![colour_light[1]; 3],
                            "ColourLight3" => colour_light[2] = csv![colour_light[2]; 3],
                            "ColourLight4" => colour_light[3] = csv![colour_light[3]; 3],
                            "ColourLight5" => colour_light[4] = csv![colour_light[4]; 3],
                            "ColourLight6" => colour_light[5] = csv![colour_light[5]; 3],
                            "ColourLight7" => colour_light[6] = csv![colour_light[6]; 3],

                            k => enumerate_match! { k,
                                "KeyImage", "" => keys_name = value.to_owned(), (0, [0 1 2 3 4 5 6]),
                                "KeyImage", "D" => keys_d_name = value.to_owned(), (0, [0 1 2 3 4 5 6]),
                                "NoteImage", "" => notes_name = value.to_owned(), (0, [0 1 2 3 4 5 6]),
                                "NoteImage", "H" => lns_head_name = value.to_owned(), (0, [0 1 2 3 4 5 6]),
                                "NoteImage", "L" => lns_body_name = value.to_owned(), (0, [0 1 2 3 4 5 6]),
                                "NoteImage", "T" => lns_tail_name = value.to_owned(), (0, [0 1 2 3 4 5 6]),
                            },
                        }
                    }
                },
            }
        }
    }

    let mut cache = HashMap::new();

    let miss = load_texture_anim(&mut cache, dir, default_dir, &miss_name, &texture_settings)?;
    let hit50 = load_texture_anim(&mut cache, dir, default_dir, &hit50_name, &texture_settings)?;
    let hit100 = load_texture_anim(&mut cache, dir, default_dir, &hit100_name, &texture_settings)?;
    let hit200 = load_texture_anim(&mut cache, dir, default_dir, &hit200_name, &texture_settings)?;
    let hit300 = load_texture_anim(&mut cache, dir, default_dir, &hit300_name, &texture_settings)?;
    let hit300g = load_texture_anim(&mut cache, dir, default_dir, &hit300g_name, &texture_settings)?;
    let stage_light = load_texture_anim(&mut cache, dir, default_dir, &stage_light_name, &texture_settings)?;
    let keys = [load_texture(&mut cache, dir, default_dir, &keys_name[0], &texture_settings)?,
                load_texture(&mut cache, dir, default_dir, &keys_name[1], &texture_settings)?,
                load_texture(&mut cache, dir, default_dir, &keys_name[2], &texture_settings)?,
                load_texture(&mut cache, dir, default_dir, &keys_name[3], &texture_settings)?,
                load_texture(&mut cache, dir, default_dir, &keys_name[4], &texture_settings)?,
                load_texture(&mut cache, dir, default_dir, &keys_name[5], &texture_settings)?,
                load_texture(&mut cache, dir, default_dir, &keys_name[6], &texture_settings)?];

    let keys_d = [load_texture(&mut cache, dir, default_dir, &keys_d_name[0], &texture_settings)?,
                  load_texture(&mut cache, dir, default_dir, &keys_d_name[1], &texture_settings)?,
                  load_texture(&mut cache, dir, default_dir, &keys_d_name[2], &texture_settings)?,
                  load_texture(&mut cache, dir, default_dir, &keys_d_name[3], &texture_settings)?,
                  load_texture(&mut cache, dir, default_dir, &keys_d_name[4], &texture_settings)?,
                  load_texture(&mut cache, dir, default_dir, &keys_d_name[5], &texture_settings)?,
                  load_texture(&mut cache, dir, default_dir, &keys_d_name[6], &texture_settings)?];

    let notes = [load_texture_anim(&mut cache, dir, default_dir, &notes_name[0], &texture_settings)?,
                 load_texture_anim(&mut cache, dir, default_dir, &notes_name[1], &texture_settings)?,
                 load_texture_anim(&mut cache, dir, default_dir, &notes_name[2], &texture_settings)?,
                 load_texture_anim(&mut cache, dir, default_dir, &notes_name[3], &texture_settings)?,
                 load_texture_anim(&mut cache, dir, default_dir, &notes_name[4], &texture_settings)?,
                 load_texture_anim(&mut cache, dir, default_dir, &notes_name[5], &texture_settings)?,
                 load_texture_anim(&mut cache, dir, default_dir, &notes_name[6], &texture_settings)?];

    let long_notes_head = [load_texture_anim(&mut cache, dir, default_dir, &lns_head_name[0], &texture_settings)?,
                           load_texture_anim(&mut cache, dir, default_dir, &lns_head_name[1], &texture_settings)?,
                           load_texture_anim(&mut cache, dir, default_dir, &lns_head_name[2], &texture_settings)?,
                           load_texture_anim(&mut cache, dir, default_dir, &lns_head_name[3], &texture_settings)?,
                           load_texture_anim(&mut cache, dir, default_dir, &lns_head_name[4], &texture_settings)?,
                           load_texture_anim(&mut cache, dir, default_dir, &lns_head_name[5], &texture_settings)?,
                           load_texture_anim(&mut cache, dir, default_dir, &lns_head_name[6], &texture_settings)?];

    let long_notes_body = [load_texture_anim(&mut cache, dir, default_dir, &lns_body_name[0], &texture_settings)?,
                           load_texture_anim(&mut cache, dir, default_dir, &lns_body_name[1], &texture_settings)?,
                           load_texture_anim(&mut cache, dir, default_dir, &lns_body_name[2], &texture_settings)?,
                           load_texture_anim(&mut cache, dir, default_dir, &lns_body_name[3], &texture_settings)?,
                           load_texture_anim(&mut cache, dir, default_dir, &lns_body_name[4], &texture_settings)?,
                           load_texture_anim(&mut cache, dir, default_dir, &lns_body_name[5], &texture_settings)?,
                           load_texture_anim(&mut cache, dir, default_dir, &lns_body_name[6], &texture_settings)?];

    let long_notes_tail = [load_texture_anim(&mut cache, dir, default_dir, &lns_tail_name[0], &texture_settings).ok(),
                           load_texture_anim(&mut cache, dir, default_dir, &lns_tail_name[1], &texture_settings).ok(),
                           load_texture_anim(&mut cache, dir, default_dir, &lns_tail_name[2], &texture_settings).ok(),
                           load_texture_anim(&mut cache, dir, default_dir, &lns_tail_name[3], &texture_settings).ok(),
                           load_texture_anim(&mut cache, dir, default_dir, &lns_tail_name[4], &texture_settings).ok(),
                           load_texture_anim(&mut cache, dir, default_dir, &lns_tail_name[5], &texture_settings).ok(),
                           load_texture_anim(&mut cache, dir, default_dir, &lns_tail_name[6], &texture_settings).ok()];

    let stage_hint = load_texture_anim(&mut cache, dir, default_dir, &stage_hint_name, &texture_settings)?;
    let stage_left = load_texture(&mut cache, dir, default_dir, &stage_left_name, &texture_settings)?;
    let stage_right = load_texture(&mut cache, dir, default_dir, &stage_right_name, &texture_settings)?;
    let stage_bottom = load_texture_anim(&mut cache, dir, default_dir, &stage_bottom_name, &texture_settings).ok();

    let smallest_note_width;
    let smallest_note_height;
    {
        let smallest_height_note = &notes.iter().min_by_key(|x| x[0].get_height()).unwrap()[0];
        smallest_note_width = smallest_height_note.get_width() as f64;
        smallest_note_height = smallest_height_note.get_height() as f64;
    }
    let width_for_note_height_scale =  smallest_note_height / smallest_note_width * *column_width.iter().min().unwrap() as f64;
    Ok(Box::new(OsuSkin {
        textures: OsuSkinTextures {
            miss,
            hit50,
            hit100,
            hit200,
            hit300,
            hit300g,
            stage_light,
            keys,
            keys_d,
            notes,
            long_notes_head,
            long_notes_body,
            long_notes_tail,
            stage_hint,
            stage_left,
            stage_right,
            stage_bottom,
        },

        anim_states: OsuAnimStates {
            keys_last_down_time: [None; 7],
        },

        config: OsuSkinConfig {
            column_start,
            column_width,
            column_spacing,
            column_line_width,
            hit_position,
            score_position,
            width_for_note_height_scale,
            note_body_style,
            colour_light,
        },
        judgement: None,
    }))
}
