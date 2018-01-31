//! Osu parser module

use std::io;
use std::io::BufRead;
use std::error;

use chart::{ Chart, ChartParser, ParseError };

/// Parses .osu charts and returns a `Chart`
pub struct OsuParser<R: io::Read> {
    reader: io::BufReader<R>,
    current_section: Option<String>,
}

impl<R: io::Read> OsuParser<R> {

    pub fn new(reader: R) -> Self {
        Self {
            reader: io::BufReader::new(reader),
            current_section: None,
        }
    }
}

impl<R: io::Read> ChartParser for OsuParser<R> {

    fn parse(mut self) -> Result<Chart, Box<error::Error>> {

        let mut buf: String = String::new();

        self.reader.read_line(&mut buf);

        let line = buf.trim();

        if !line.starts_with("osu file format v") {
            return Err(Box::new(ParseError::InvalidFile));
        }

        let version: i32 = line[17..].parse()?;

        println!("Version {}", version);

        Ok(Chart::default())
    }
}
