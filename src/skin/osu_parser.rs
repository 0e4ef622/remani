//! Osu skin directory parser module

extern crate opengl_graphics;
extern crate texture;

use opengl_graphics::Texture;
use std::io;
use std::io::BufRead;
use std::path;
use std::fs;
use std::rc::Rc;
use self::texture::TextureSettings;

use skin::{ Skin, SkinParser, ParseError };

/// Loads osu skin images from directory and returns a `Skin`
pub struct OsuParser {
    dir: path::PathBuf,
}

impl OsuParser {

    /// Create a new parser
    pub fn new(dir: path::PathBuf) -> Self {
        Self {
            dir: dir,
        }
    }
}

impl SkinParser for OsuParser {

    fn parse(self) -> Result<Skin, ParseError> {
        // TODO: read configuration file
        let config_path = self.dir.join(path::Path::new("skin.ini"));
        let mut skin = Skin::default();

        // test
        skin.miss.push(Texture::from_path(self.dir.join("mania-hit0.png").as_path(), &TextureSettings::new()).unwrap());
        skin.hit50.push(Texture::from_path(self.dir.join("mania-hit50.png").as_path(), &TextureSettings::new()).unwrap());
        skin.hit100.push(Texture::from_path(self.dir.join("mania-hit100.png").as_path(), &TextureSettings::new()).unwrap());
        skin.hit300.push(Texture::from_path(self.dir.join("mania-hit300.png").as_path(), &TextureSettings::new()).unwrap());
        skin.hit300g.push(Texture::from_path(self.dir.join("mania-hit300g-0.png").as_path(), &TextureSettings::new()).unwrap());

        let key1 = Rc::new(Texture::from_path(self.dir.join("mania-key1.png").as_path(), &TextureSettings::new()).unwrap());
        let key2 = Rc::new(Texture::from_path(self.dir.join("mania-key2.png").as_path(), &TextureSettings::new()).unwrap());
        let key3 = Rc::new(Texture::from_path(self.dir.join("mania-keyS.png").as_path(), &TextureSettings::new()).unwrap());
        skin.keys[0].push(key1.clone());
        skin.keys[1].push(key2.clone());
        skin.keys[2].push(key1.clone());
        skin.keys[3].push(key3.clone());
        skin.keys[4].push(key1.clone());
        skin.keys[5].push(key2.clone());
        skin.keys[6].push(key1.clone());

        let key1_d = Rc::new(Texture::from_path(self.dir.join("mania-key1D.png").as_path(), &TextureSettings::new()).unwrap());
        let key2_d = Rc::new(Texture::from_path(self.dir.join("mania-key2D.png").as_path(), &TextureSettings::new()).unwrap());
        let key3_d = Rc::new(Texture::from_path(self.dir.join("mania-keySD.png").as_path(), &TextureSettings::new()).unwrap());
        skin.keys_d[0].push(key1_d.clone());
        skin.keys_d[1].push(key2_d.clone());
        skin.keys_d[2].push(key1_d.clone());
        skin.keys_d[3].push(key3_d.clone());
        skin.keys_d[4].push(key1_d.clone());
        skin.keys_d[5].push(key2_d.clone());
        skin.keys_d[6].push(key1_d.clone());

        let note1 = Rc::new(Texture::from_path(self.dir.join("mania-note1.png").as_path(), &TextureSettings::new()).unwrap());
        let note2 = Rc::new(Texture::from_path(self.dir.join("mania-note2.png").as_path(), &TextureSettings::new()).unwrap());
        let note3 = Rc::new(Texture::from_path(self.dir.join("mania-noteS.png").as_path(), &TextureSettings::new()).unwrap());
        skin.notes[0].push(note1.clone());
        skin.notes[1].push(note2.clone());
        skin.notes[2].push(note1.clone());
        skin.notes[3].push(note3.clone());
        skin.notes[4].push(note1.clone());
        skin.notes[5].push(note2.clone());
        skin.notes[6].push(note1.clone());

        let ln1_head = Rc::new(Texture::from_path(self.dir.join("mania-note1H.png").as_path(), &TextureSettings::new()).unwrap());
        let ln2_head = Rc::new(Texture::from_path(self.dir.join("mania-note2H.png").as_path(), &TextureSettings::new()).unwrap());
        let ln3_head = Rc::new(Texture::from_path(self.dir.join("mania-noteSH.png").as_path(), &TextureSettings::new()).unwrap());
        skin.long_notes_head[0].push(ln1_head.clone());
        skin.long_notes_head[1].push(ln2_head.clone());
        skin.long_notes_head[2].push(ln1_head.clone());
        skin.long_notes_head[3].push(ln3_head.clone());
        skin.long_notes_head[4].push(ln1_head.clone());
        skin.long_notes_head[5].push(ln2_head.clone());
        skin.long_notes_head[6].push(ln1_head.clone());

        let ln1_body = Rc::new(Texture::from_path(self.dir.join("mania-note1L-0.png").as_path(), &TextureSettings::new()).unwrap());
        let ln2_body = Rc::new(Texture::from_path(self.dir.join("mania-note2L-0.png").as_path(), &TextureSettings::new()).unwrap());
        let ln3_body = Rc::new(Texture::from_path(self.dir.join("mania-noteSL-0.png").as_path(), &TextureSettings::new()).unwrap());
        skin.long_notes_body[0].push(ln1_body.clone());
        skin.long_notes_body[1].push(ln2_body.clone());
        skin.long_notes_body[2].push(ln1_body.clone());
        skin.long_notes_body[3].push(ln3_body.clone());
        skin.long_notes_body[4].push(ln1_body.clone());
        skin.long_notes_body[5].push(ln2_body.clone());
        skin.long_notes_body[6].push(ln1_body.clone());
        // end test

        // default values
        skin.column_start = 136;
        skin.column_width = 30;
        skin.column_line_width = 2;
        skin.hit_position = 402;

        if config_path.exists() {
            /* let general_section_name = "__General__".into();
            let config = Ini::load_from_file(config_path).unwrap();
            for (sec, prop) in config.iter() {
                let section_name = sec.as_ref().unwrap_or(&general_section_name);
                println!("-- Section: {:?} begins", section_name);
                for (k, v) in prop.iter() {
                    println!("{}: {:?}", *k, *v);
                    match section_name.as_ref() {
                        "Mania" => {
                            match k.as_ref() {
                                "ColumnStart" => skin.column_start = v.parse::<u16>().unwrap(),
                                "ColumnWidth" => skin.column_width = v.parse::<u16>().unwrap(),
                                "ColumnLineWidth" => skin.column_line_width = v.parse::<u16>().unwrap(),
                                "HitPosition" => skin.hit_position = v.parse::<u16>().unwrap(),
                                _ => (),
                            }
                        },

                        _ => (),
                    }
                }
            } */
        }
        Ok(skin)
    }
}
