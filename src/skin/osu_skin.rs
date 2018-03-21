//! Osu skin directory parser module

use image;

use opengl_graphics::Texture;
use opengl_graphics::GlGraphics;
use graphics::image::Image;
use graphics::draw_state::DrawState;
use graphics::math;
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

use skin::{ Skin, ParseError };

/// Holds skin data, such as note images and what not.
struct OsuSkin {
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

    /// The stage components.
    stage_hint: Rc<Vec<Rc<Texture>>>,
    stage_left: Rc<Texture>,
    stage_right: Rc<Texture>,

    /// Various information related to how to draw components.
    column_start: u16,
    column_width: Vec<u16>,
    column_line_width: Vec<u16>,
    hit_position: u16,
    width_for_note_height_scale: f64,
}

impl Skin for OsuSkin {
    fn draw_note(&self, draw_state: &DrawState, transform: math::Matrix2d, gl: &mut GlGraphics, y_pos: f64, column_index: usize) {

        let stage_h = 480.0;
        let scale = stage_h / 480.0;

        // ar = aspect ratio
        let column_start = self.column_start as f64 * scale;

        let note = self.notes[column_index][0].deref();
        let note_img = Image::new().rect([column_start + self.column_width[0..column_index].iter().sum::<u16>() as f64 * scale, y_pos, self.column_width[column_index] as f64 * scale, self.width_for_note_height_scale * scale]);
        note_img.draw(note, draw_state, transform, gl);
    }
    fn draw_track(&self, draw_state: &DrawState, transform: math::Matrix2d, gl: &mut GlGraphics) {

        let stage_h = 480.0;
        let scale = stage_h / 480.0;

        // ar = aspect ratio
        let stage_l_ar = self.stage_left.get_width() as f64 / self.stage_left.get_height() as f64;
        let stage_r_ar = self.stage_right.get_width() as f64 / self.stage_right.get_height() as f64;

        let column_width_sum = self.column_width.iter().sum::<u16>() as f64 * scale;
        let column_start = self.column_start as f64 * scale;
        let stage_hint_height = self.stage_hint[0].get_height() as f64;
        let stage_l_scaled_width = stage_l_ar * stage_h;
        let stage_r_scaled_width = stage_r_ar * stage_h;

        let stage_l_img = Image::new().rect([column_start - stage_l_scaled_width , 0.0, stage_l_scaled_width, stage_h]);
        let stage_r_img = Image::new().rect([column_start + column_width_sum, 0.0, stage_r_scaled_width, stage_h]);
        let stage_hint_img = Image::new().rect([column_start, self.hit_position as f64 * scale - stage_hint_height / 2.0, column_width_sum, stage_hint_height]);

        stage_hint_img.draw(self.stage_hint[0].deref(), draw_state, transform, gl);
        stage_l_img.draw(self.stage_left.deref(), draw_state, transform, gl);
        stage_r_img.draw(self.stage_right.deref(), draw_state, transform, gl);
    }
    fn draw_keys(&self, draw_state: &DrawState, transform: math::Matrix2d, gl: &mut GlGraphics, pressed: &[bool]) {

        let stage_h = 480.0;
        let scale = stage_h / 480.0;

        for (i, key_pressed) in pressed.iter().enumerate() {
            let key_texture: &Texture = if *key_pressed { self.keys_d[i].as_ref() } else { self.keys[i].as_ref() };
            let key_width = self.column_width[i] as f64 * scale;

            // Seriously peppy?
            let key_height = key_texture.get_height() as f64 * scale * 480.0 / 768.0;
            let key_x = scale * (self.column_start as f64 + self.column_width[0..i].iter().sum::<u16>() as f64);
            let key_y = stage_h - key_height;
            let key_img = Image::new().rect([key_x, key_y, key_width, key_height]);
            key_img.draw(key_texture, draw_state, transform, gl);
        }
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

    if cache.contains_key(&names.1) {
        return Ok(Rc::clone(cache.get(&names.1).unwrap()));
    } else if cache.contains_key(names.0) {
        return Ok(Rc::clone(cache.get(names.0).unwrap()));
    }

    let mut textures = Vec::new();
    let mut path;

    macro_rules! repetitive_code {
        ($(($dir:ident, $name:expr)),*) => {$(
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

    // rust devs pls fix borrow checker
    if cache.contains_key(names.0) {
        return Ok(Rc::clone(&cache.get(names.0).unwrap()[0]));
    }

    if cache.contains_key(&names.1) {
        return Ok(Rc::clone(&cache.get(&names.1).unwrap()[0]));
    }

    macro_rules! repetitive_code {
        ($(($dir:ident, $name:expr)),*) => {$(
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

    // TODO stage_bottom
    let mut stage_hint_name = double!("mania-stage-hint");
    let mut stage_left_name = double!("mania-stage-left");
    let mut stage_right_name = double!("mania-stage-right");

    // default values
    let mut column_start = 136;
    let mut column_width = vec!(30, 30, 30, 30, 30, 30, 30);
    let mut column_line_width = vec!(2, 2, 2, 2, 2, 2, 2, 2);
    let mut hit_position = 402;

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
                        match key {
                            "ColumnStart" => column_start = value.parse().unwrap(),
                            "HitPosition" => hit_position = value.parse().unwrap(),
                            "ColumnWidth" => {
                                let number_strings: Vec<&str> = value.split(",").collect();
                                for (i, number_string) in number_strings.iter().enumerate() {
                                    column_width[i] = number_string.parse().unwrap();
                                }
                            },
                            "ColumnLineWidth" => {
                                let number_strings: Vec<&str> = value.split(",").collect();
                                for (i, number_string) in number_strings.iter().enumerate() {
                                    column_line_width[i] = number_string.parse().unwrap();
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

    // TODO streamline this an bit more ;-;
    let miss = load_texture_anim(&mut cache, dir, default_dir, &miss_name, &texture_settings)?;
    let hit50 = load_texture_anim(&mut cache, dir, default_dir, &hit50_name, &texture_settings)?;
    let hit100 = load_texture_anim(&mut cache, dir, default_dir, &hit100_name, &texture_settings)?;
    let hit200 = load_texture_anim(&mut cache, dir, default_dir, &hit200_name, &texture_settings)?;
    let hit300 = load_texture_anim(&mut cache, dir, default_dir, &hit300_name, &texture_settings)?;
    let hit300g = load_texture_anim(&mut cache, dir, default_dir, &hit300g_name, &texture_settings)?;
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
    let stage_hint = load_texture_anim(&mut cache, dir, default_dir, &stage_hint_name, &texture_settings)?;
    let stage_left = load_texture(&mut cache, dir, default_dir, &stage_left_name, &texture_settings)?;
    let stage_right = load_texture(&mut cache, dir, default_dir, &stage_right_name, &texture_settings)?;

    let smallest_note_width;
    let smallest_note_height;
    {
        let smallest_height_note = &notes.iter().min_by_key(|x| x[0].get_height()).unwrap()[0];
        smallest_note_width = smallest_height_note.get_width() as f64;
        smallest_note_height = smallest_height_note.get_height() as f64;
    }
    let width_for_note_height_scale =  smallest_note_height / smallest_note_width * *column_width.iter().min().unwrap() as f64;
    Ok(Box::new(OsuSkin {
        miss,
        hit50,
        hit100,
        hit200,
        hit300,
        hit300g,
        keys,
        keys_d,
        notes,
        long_notes_head,
        long_notes_body,
        stage_hint,
        stage_left,
        stage_right,
        column_start,
        column_width,
        column_line_width,
        hit_position,
        width_for_note_height_scale,
    }))
}
