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

    /// Create a new parser
    pub fn new(reader: R) -> Self {
        Self {
            reader: io::BufReader::new(reader),
            current_section: None,
        }
    }

    /// Runs first, verifies that the file headers are correct
    fn verify(&mut self) -> Result<i32, ParseError> {
        let mut buf: String = String::new();

        self.reader.read_line(&mut buf);

        let line = buf.trim();

        if !line.starts_with("osu file format v") {
            return Err(ParseError::InvalidFile);
        }

        match line[17..].parse::<i32>() {
            Ok(n) => Ok(n),
            Err(e) => Err(
                ParseError::Parse(String::from("Error parsing file version"),
                Some(Box::new(e)))),
        }
    }
}

impl<R: io::Read> ChartParser for OsuParser<R> {

    fn parse(mut self) -> Result<Chart, ParseError> {

        let version = self.verify()?;

        println!("Version {}", version);

        Ok(Chart::default())
    }
}
