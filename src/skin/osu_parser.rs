//! Osu skin directory parser module

use std::io;
use std::io::BufRead;
use std::path;

use skin::{ Skin, SkinParser, ParseError };

/// Loads osu skin images from directory and returns a `Skin`
pub struct OsuParser<P: io::path::Path> {
    dir: io::path::Path<P>,
}

impl<P: path::Path> OsuParser<P> {

    /// Create a new parser
    pub fn new(dir: P) -> Self {
        Self {
            dir: P,
        }
    }
}

impl<P: path::Path> SkinParser for OsuParser<P> {

    fn parse(self) -> Result<Skin, ParseError> {
        // TODO: read configuration file
        let configPath = dir.join(Path::new("config.ini"));
        let skin = Skin::default;
        if (configPath.exists()) {
            let reader = BufReader::new(File::open(configPath));
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
