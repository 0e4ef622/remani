//! Osu skin directory parser module

use std::io;
use std::path;

use chart::{ Chart, ChartParser, ParseError };

/// Loads osu skin images from directory and returns a `Skin`
pub struct OsuParser<P: io::path::Path> {
    dir: io::path::Path<P>,
    current_section: String,
}

impl<R: path::Path> OsuParser<R> {

    pub fn new(dir: P) -> Self {
        Self {
            dir: P,
            current_section: String::new(),
        }
    }
}

impl<P: path::Path> SkinParser for OsuParser<R> {

    // TODO: read configuration file
    fn parse(self) -> Result<Skin, ParseError> {
        for line in self.reader.lines() {
            let line = line?;
            let line = line.trim();
            match line {
                "[General]" => println!("Found General section"),
                _ => (),
            }
        }

        Ok(Chart::default())
    }
}
