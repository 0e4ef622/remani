//! Osu skin directory parser module

use std::io;
use std::io::BufRead;
use std::path;
use std::fs;

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
        let configPath = self.dir.join(path::Path::new("skin.ini"));
        let skin = Skin::default();
        if configPath.exists() {
            let reader = io::BufReader::new(fs::File::open(configPath)?);
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
