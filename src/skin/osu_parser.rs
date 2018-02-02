//! Osu skin directory parser module

extern crate opengl_graphics;
extern crate texture;

use opengl_graphics::Texture;

use std::io;
use std::io::BufRead;
use std::path;
use std::fs;
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
        skin.mania_hit.push(Texture::from_path(self.dir.join("mania-hit0.png").as_path(), &TextureSettings::new()).unwrap());

        if config_path.exists() {
            let reader = io::BufReader::new(fs::File::open(config_path)?);
            for line in reader.lines() {
                let line = line?;
                let line = line.trim();
                match line {
                    "[General]" => println!("Found General section"),
                    _ => (),
                }
            }
        }
        Ok(skin)
    }
}
