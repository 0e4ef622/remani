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
        skin.miss.push(Texture::from_path(self.dir.join("mania-hit0.png").as_path(), &TextureSettings::new()).unwrap());
        skin.hit50.push(Texture::from_path(self.dir.join("mania-hit50.png").as_path(), &TextureSettings::new()).unwrap());
        skin.hit100.push(Texture::from_path(self.dir.join("mania-hit100.png").as_path(), &TextureSettings::new()).unwrap());
        skin.hit300.push(Texture::from_path(self.dir.join("mania-hit300.png").as_path(), &TextureSettings::new()).unwrap());
        skin.hit300g.push(Texture::from_path(self.dir.join("mania-hit300g-0.png").as_path(), &TextureSettings::new()).unwrap());

        skin.key1.push(Texture::from_path(self.dir.join("mania-key1.png").as_path(), &TextureSettings::new()).unwrap());
        skin.key2.push(Texture::from_path(self.dir.join("mania-key2.png").as_path(), &TextureSettings::new()).unwrap());
        skin.key3.push(Texture::from_path(self.dir.join("mania-keyS.png").as_path(), &TextureSettings::new()).unwrap());

        skin.key1D.push(Texture::from_path(self.dir.join("mania-key1D.png").as_path(), &TextureSettings::new()).unwrap());
        skin.key2D.push(Texture::from_path(self.dir.join("mania-key2D.png").as_path(), &TextureSettings::new()).unwrap());
        skin.key3D.push(Texture::from_path(self.dir.join("mania-keySD.png").as_path(), &TextureSettings::new()).unwrap());

        skin.note1.push(Texture::from_path(self.dir.join("mania-note1.png").as_path(), &TextureSettings::new()).unwrap());
        skin.note2.push(Texture::from_path(self.dir.join("mania-note2.png").as_path(), &TextureSettings::new()).unwrap());
        skin.note3.push(Texture::from_path(self.dir.join("mania-noteS.png").as_path(), &TextureSettings::new()).unwrap());

        skin.note1H.push(Texture::from_path(self.dir.join("mania-note1H.png").as_path(), &TextureSettings::new()).unwrap());
        skin.note2H.push(Texture::from_path(self.dir.join("mania-note2H.png").as_path(), &TextureSettings::new()).unwrap());
        skin.note3H.push(Texture::from_path(self.dir.join("mania-noteSH.png").as_path(), &TextureSettings::new()).unwrap());

        skin.note1L.push(Texture::from_path(self.dir.join("mania-note1L-0.png").as_path(), &TextureSettings::new()).unwrap());
        skin.note2L.push(Texture::from_path(self.dir.join("mania-note2L-0.png").as_path(), &TextureSettings::new()).unwrap());
        skin.note3L.push(Texture::from_path(self.dir.join("mania-noteSL-0.png").as_path(), &TextureSettings::new()).unwrap());
        // end test

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
