//! Osu parser module

use std::io;
use std::io::BufRead;

use chart::{ Chart, ChartParser, ParseError };

/// Parses .osu charts and returns a `Chart`
pub struct OsuParser<R: io::Read> {
    reader: io::BufReader<R>,
    current_section: String,
}

impl<R: io::Read> OsuParser<R> {

    pub fn new(reader: R) -> Self {
        Self {
            reader: io::BufReader::new(reader),
            current_section: String::new(),
        }
    }
}

impl<R: io::Read> ChartParser for OsuParser<R> {

    fn parse(self) -> Result<Chart, ParseError> {
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
