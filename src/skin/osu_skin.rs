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
use std::path;
use std::fs::File;
use std::rc::Rc;

use skin::{ Skin, ParseError };

/// Holds skin data, such as note images and what not.
struct OsuSkin {
    miss: Vec<Texture>,
    hit50: Vec<Texture>,
    hit100: Vec<Texture>,
    hit300: Vec<Texture>,
    hit300g: Vec<Texture>,

    /// The images virtual keys under the judgement line.
    keys: [Rc<Texture>; 7],

    /// The images of the virtual keys under the judgement line when the
    /// corresponding key on the keyboard is pressed.
    keys_d: [Rc<Texture>; 7],

    /// The notes' images.
    notes: [Vec<Rc<Texture>>; 7],

    /// The long notes' ends' images.
    long_notes_head: [Vec<Rc<Texture>>; 7],

    /// The long notes' bodies' images.
    long_notes_body: [Vec<Rc<Texture>>; 7],

    /// The stage components.
    stage_hint: Rc<Texture>,
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
        let stage_hint_height = self.stage_hint.get_height() as f64;
        let stage_l_scaled_width = stage_l_ar * stage_h;
        let stage_r_scaled_width = stage_r_ar * stage_h;

        let stage_l_img = Image::new().rect([column_start - stage_l_scaled_width , 0.0, stage_l_scaled_width, stage_h]);
        let stage_r_img = Image::new().rect([column_start + column_width_sum, 0.0, stage_r_scaled_width, stage_h]);
        let stage_hint_img = Image::new().rect([column_start, self.hit_position as f64 * scale - stage_hint_height / 2.0, column_width_sum, stage_hint_height]);

        stage_hint_img.draw(self.stage_hint.deref(), draw_state, transform, gl);
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

    let texture_settings = TextureSettings::new();

    // test
    let miss = vec![texture_from_path(dir.join("mania-hit0.png"), &texture_settings)];
    let hit50 = vec![texture_from_path(dir.join("mania-hit50.png"), &texture_settings)];
    let hit100 = vec![texture_from_path(dir.join("mania-hit100.png"), &texture_settings)];
    let hit300 = vec![texture_from_path(dir.join("mania-hit300.png"), &texture_settings)];
    let hit300g = vec![texture_from_path(dir.join("mania-hit300g-0.png"), &texture_settings)];

    let key1 = Rc::new(texture_from_path(dir.join("mania-key1.png"), &texture_settings));
    let key2 = Rc::new(texture_from_path(dir.join("mania-key2.png"), &texture_settings));
    let key3 = Rc::new(texture_from_path(dir.join("mania-keyS.png"), &texture_settings));
    let keys = [key1.clone(),
                key2.clone(),
                key1.clone(),
                key3.clone(),
                key1.clone(),
                key2.clone(),
                key1.clone()];

    let key1_d = Rc::new(texture_from_path(dir.join("mania-key1D.png"), &texture_settings));
    let key2_d = Rc::new(texture_from_path(dir.join("mania-key2D.png"), &texture_settings));
    let key3_d = Rc::new(texture_from_path(dir.join("mania-keySD.png"), &texture_settings));
    let keys_d = [key1_d.clone(),
                  key2_d.clone(),
                  key1_d.clone(),
                  key3_d.clone(),
                  key1_d.clone(),
                  key2_d.clone(),
                  key1_d.clone()];

    let note1 = Rc::new(texture_from_path(dir.join("mania-note1.png"), &texture_settings));
    let note2 = Rc::new(texture_from_path(dir.join("mania-note2.png"), &texture_settings));
    let note3 = Rc::new(texture_from_path(dir.join("mania-noteS.png"), &texture_settings));
    let notes = [vec![note1.clone()],
                 vec![note2.clone()],
                 vec![note1.clone()],
                 vec![note3.clone()],
                 vec![note1.clone()],
                 vec![note2.clone()],
                 vec![note1.clone()]];

    let ln1_head = Rc::new(texture_from_path(dir.join("mania-note1H.png"), &texture_settings));
    let ln2_head = Rc::new(texture_from_path(dir.join("mania-note2H.png"), &texture_settings));
    let ln3_head = Rc::new(texture_from_path(dir.join("mania-noteSH.png"), &texture_settings));
    let long_notes_head = [vec![ln1_head.clone()],
                           vec![ln2_head.clone()],
                           vec![ln1_head.clone()],
                           vec![ln3_head.clone()],
                           vec![ln1_head.clone()],
                           vec![ln2_head.clone()],
                           vec![ln1_head.clone()]];

    let ln1_body = Rc::new(texture_from_path(dir.join("mania-note1L-0.png"), &texture_settings));
    let ln2_body = Rc::new(texture_from_path(dir.join("mania-note2L-0.png"), &texture_settings));
    let ln3_body = Rc::new(texture_from_path(dir.join("mania-noteSL-0.png"), &texture_settings));
    let long_notes_body = [vec![ln1_body.clone()],
                           vec![ln2_body.clone()],
                           vec![ln1_body.clone()],
                           vec![ln3_body.clone()],
                           vec![ln1_body.clone()],
                           vec![ln2_body.clone()],
                           vec![ln1_body.clone()]];

    let stage_hint = Rc::new(texture_from_path(dir.join("mania-stage-hint.png"), &texture_settings));
    let stage_left = Rc::new(texture_from_path(dir.join("mania-stage-left.png"), &texture_settings));
    let stage_right = Rc::new(texture_from_path(dir.join("mania-stage-right.png"), &texture_settings));
    // end test

    // default values
    let mut column_start = 136;
    let mut column_width = vec!(30, 30, 30, 30, 30, 30, 30);
    let mut column_line_width = vec!(2, 2, 2, 2, 2, 2, 2, 2);
    let mut hit_position = 402;

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
                            _ => (),
                        }
                    }
                },
            }
        }
    }
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
