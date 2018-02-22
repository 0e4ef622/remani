//! Osu skin directory parser module

extern crate opengl_graphics;
extern crate texture;
extern crate image;

use opengl_graphics::Texture;
use opengl_graphics::GlGraphics;
use graphics::image::Image;
use graphics::draw_state::DrawState;
use graphics::math;
use self::texture::TextureSettings;
use view::texture::ImageSize;
use std::ops::Deref;

use std::io::BufReader;
use std::io::BufRead;
use std::path;
use std::fs::File;
use std::rc::Rc;

use skin::{ Skin, ParseError };

/// Holds skin data, such as note images and what not.
#[derive(Default)]
struct OsuSkin {
    miss: Vec<Texture>,
    hit50: Vec<Texture>,
    hit100: Vec<Texture>,
    hit300: Vec<Texture>,
    hit300g: Vec<Texture>,

    /// The images virtual keys under the judgement line.
    keys: [Vec<Rc<Texture>>; 7],

    /// The images of the virtual keys under the judgement line when the
    /// corresponding key on the keyboard is pressed.
    keys_d: [Vec<Rc<Texture>>; 7],

    /// The notes' images.
    notes: [Vec<Rc<Texture>>; 7],

    /// The long notes' ends' images.
    long_notes_head: [Vec<Rc<Texture>>; 7],

    /// The long notes' bodies' images.
    long_notes_body: [Vec<Rc<Texture>>; 7],

    /// The stage components.
    stage_hint: Option<Rc<Texture>>,
    stage_left: Option<Rc<Texture>>,
    stage_right: Option<Rc<Texture>>,

    /// Various information related to how to draw components.
    column_start: u16,
    column_width: Vec<u16>,
    column_line_width: Vec<u16>,
    hit_position: u16,
}

impl Skin for OsuSkin {
    fn draw_stage(&self, draw_state: &DrawState, transform: math::Matrix2d, gl: &mut GlGraphics) {

        let stage_h = 480.0;

        // ar = aspect ratio
        let stage_l = self.stage_left.as_ref().unwrap().deref();
        let stage_r = self.stage_right.as_ref().unwrap().deref();
        let stage_hint = self.stage_hint.as_ref().unwrap().deref();

        let stage_l_ar = stage_l.get_width() as f64 / stage_l.get_height() as f64;
        let stage_r_ar = stage_r.get_width() as f64 / stage_r.get_height() as f64;

        let column_width_sum: u16 = self.column_width.iter().sum();
        let column_start = self.column_start as f64;
        let stage_hint_width = stage_hint.get_width() as f64;
        let stage_hint_height = stage_hint.get_height() as f64;
        let stage_l_scaled_width = stage_l_ar * stage_h;

        let stage_l_img = Image::new().rect([column_start, 0.0, stage_l_scaled_width, stage_h]);
        let stage_r_img = Image::new().rect([column_start + stage_l_scaled_width + column_width_sum as f64, 0.0, stage_r_ar * stage_h, stage_h]);
        let stage_hint_img = Image::new().rect([column_start + stage_l_scaled_width, self.hit_position as f64 - stage_hint_height / 2.0, column_width_sum as f64, stage_hint_height]);

        stage_hint_img.draw(stage_hint, draw_state, transform, gl);
        stage_l_img.draw(stage_l, draw_state, transform, gl);
        stage_r_img.draw(stage_r, draw_state, transform, gl);
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

fn texture_from_path<T: AsRef<path::Path>>(path: T, texture_settings: &TextureSettings) -> Texture {
    Texture::from_image(
        &image_reverse_srgb(image::open(path).expect("Could not load image").to_rgba()),
        texture_settings)
}

pub fn from_path(dir: &path::Path) -> Result<Box<Skin>, ParseError> {
    let config_path = dir.join(path::Path::new("skin.ini"));
    let mut skin = Box::new(OsuSkin::default());

    let texture_settings = TextureSettings::new();

    // test
    skin.miss.push(texture_from_path(dir.join("mania-hit0.png"), &texture_settings));
    skin.hit50.push(texture_from_path(dir.join("mania-hit50.png"), &texture_settings));
    skin.hit100.push(texture_from_path(dir.join("mania-hit100.png"), &texture_settings));
    skin.hit300.push(texture_from_path(dir.join("mania-hit300.png"), &texture_settings));
    skin.hit300g.push(texture_from_path(dir.join("mania-hit300g-0.png"), &texture_settings));

    let key1 = Rc::new(texture_from_path(dir.join("mania-key1.png"), &texture_settings));
    let key2 = Rc::new(texture_from_path(dir.join("mania-key2.png"), &texture_settings));
    let key3 = Rc::new(texture_from_path(dir.join("mania-keyS.png"), &texture_settings));
    skin.keys[0].push(key1.clone());
    skin.keys[1].push(key2.clone());
    skin.keys[2].push(key1.clone());
    skin.keys[3].push(key3.clone());
    skin.keys[4].push(key1.clone());
    skin.keys[5].push(key2.clone());
    skin.keys[6].push(key1.clone());

    let key1_d = Rc::new(texture_from_path(dir.join("mania-key1D.png"), &texture_settings));
    let key2_d = Rc::new(texture_from_path(dir.join("mania-key2D.png"), &texture_settings));
    let key3_d = Rc::new(texture_from_path(dir.join("mania-keySD.png"), &texture_settings));
    skin.keys_d[0].push(key1_d.clone());
    skin.keys_d[1].push(key2_d.clone());
    skin.keys_d[2].push(key1_d.clone());
    skin.keys_d[3].push(key3_d.clone());
    skin.keys_d[4].push(key1_d.clone());
    skin.keys_d[5].push(key2_d.clone());
    skin.keys_d[6].push(key1_d.clone());

    let note1 = Rc::new(texture_from_path(dir.join("mania-note1.png"), &texture_settings));
    let note2 = Rc::new(texture_from_path(dir.join("mania-note2.png"), &texture_settings));
    let note3 = Rc::new(texture_from_path(dir.join("mania-noteS.png"), &texture_settings));
    skin.notes[0].push(note1.clone());
    skin.notes[1].push(note2.clone());
    skin.notes[2].push(note1.clone());
    skin.notes[3].push(note3.clone());
    skin.notes[4].push(note1.clone());
    skin.notes[5].push(note2.clone());
    skin.notes[6].push(note1.clone());

    let ln1_head = Rc::new(texture_from_path(dir.join("mania-note1H.png"), &texture_settings));
    let ln2_head = Rc::new(texture_from_path(dir.join("mania-note2H.png"), &texture_settings));
    let ln3_head = Rc::new(texture_from_path(dir.join("mania-noteSH.png"), &texture_settings));
    skin.long_notes_head[0].push(ln1_head.clone());
    skin.long_notes_head[1].push(ln2_head.clone());
    skin.long_notes_head[2].push(ln1_head.clone());
    skin.long_notes_head[3].push(ln3_head.clone());
    skin.long_notes_head[4].push(ln1_head.clone());
    skin.long_notes_head[5].push(ln2_head.clone());
    skin.long_notes_head[6].push(ln1_head.clone());

    let ln1_body = Rc::new(texture_from_path(dir.join("mania-note1L-0.png"), &texture_settings));
    let ln2_body = Rc::new(texture_from_path(dir.join("mania-note2L-0.png"), &texture_settings));
    let ln3_body = Rc::new(texture_from_path(dir.join("mania-noteSL-0.png"), &texture_settings));
    skin.long_notes_body[0].push(ln1_body.clone());
    skin.long_notes_body[1].push(ln2_body.clone());
    skin.long_notes_body[2].push(ln1_body.clone());
    skin.long_notes_body[3].push(ln3_body.clone());
    skin.long_notes_body[4].push(ln1_body.clone());
    skin.long_notes_body[5].push(ln2_body.clone());
    skin.long_notes_body[6].push(ln1_body.clone());

    skin.stage_hint = Some(Rc::new(texture_from_path(dir.join("mania-stage-hint.png"), &texture_settings)));
    skin.stage_left = Some(Rc::new(texture_from_path(dir.join("mania-stage-left.png"), &texture_settings)));
    skin.stage_right = Some(Rc::new(texture_from_path(dir.join("mania-stage-right.png"), &texture_settings)));
    // end test

    // default values
    skin.column_start = 136;
    skin.column_width = vec!(30, 30, 30, 30, 30, 30, 30);
    skin.column_line_width = vec!(2, 2, 2, 2, 2, 2, 2);
    skin.hit_position = 402;

    if config_path.exists() {
        let config_file = File::open(config_path).unwrap();
        let config_reader = BufReader::new(&config_file);
        let mut section = String::from("General");
        let mut keys: u8 = 0;
        for l in config_reader.lines() {
            let line = l.unwrap().to_string().clone().to_owned().trim_matches(' ').to_owned();
            if line.starts_with("[") && line.ends_with("]") {
                section = line.clone();
                section = section[1..section.len()-1].to_string();
                continue;
            }
            if line.starts_with("//") || line == "" {
                continue;
            }
            let line_parts: Vec<&str> = line.splitn(2, ":").collect();
            let key = line_parts[0].trim_matches(' ');
            let value = line_parts[1].trim_matches(' ');
            match key {
                "Keys" => keys = value.parse().unwrap(),
                _ => {
                    if keys == 7 {
                        match key {
                            "ColumnStart" => skin.column_start = value.parse().unwrap(),
                            "HitPosition" => skin.hit_position = value.parse().unwrap(),
                            "ColumnWidth" => {
                                skin.column_width = Vec::with_capacity(7);
                                let number_strings: Vec<&str> = value.split(",").collect();
                                for number_string in number_strings {
                                    skin.column_width.push(number_string.parse().unwrap());
                                }
                            },
                            "ColumnLineWidth" => {
                                skin.column_line_width = Vec::with_capacity(8);
                                let number_strings: Vec<&str> = value.split(",").collect();
                                for number_string in number_strings {
                                    skin.column_line_width.push(number_string.parse().unwrap());
                                }
                            },
                            _ => (),
                        }
                    }
                },
            }
        }
    }
    Ok(skin)
}
