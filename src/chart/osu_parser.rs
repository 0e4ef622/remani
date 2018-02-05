//! Osu parser module

use std::io;
use std::io::BufRead;

use chart::{ Chart, ChartParser, ParseError };

/// Parses .osu charts and returns a `Chart`
pub struct OsuParser<R: io::Read> {
    reader: io::BufReader<R>,
}

impl<R: io::Read> OsuParser<R> {

    /// Create a new parser
    pub fn new(reader: R) -> Self {
        Self {
            reader: io::BufReader::new(reader),
        }
    }

    /// Calls `read_line` on `reader` with `current_line` as the string buffer
    pub fn read_line(&mut self) -> Result<String, ParseError> {
        loop {
            let mut line = String::new();
            match self.reader.read_line(&mut line) {
                Err(e) => return Err(
                    ParseError::Io(String::from("Error reading chart"), e)),
                _ => (),
            }
            if line.len() == 0 {
                return Err(ParseError::EOF);
            }
            let trim = line.trim();
            if trim.len() > 0 {
                return Ok(String::from(line.trim()));
            }
        }
    }

    /// Runs first, verifies that the file headers are correct
    fn verify(&mut self) -> Result<i32, ParseError> {

        let line = self.read_line()?;

        let line = line.trim();

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

    /// Finds and returns the next section
    fn next_section(&mut self) -> Result<String, ParseError> {
        loop {
            let line = match self.read_line() {
                Ok(s) => s,
                Err(ParseError::EOF) => return Err(ParseError::Parse(
                        String::from("Error finding next section"),
                        Some(Box::new(ParseError::EOF)))),
                e => return e
            };
            let line = line.trim();
            if &line[0..1] == "[" {
                return Ok(String::from(&line[1..line.len()-1]));
            }
        }
    }
}

impl<R: io::Read> ChartParser for OsuParser<R> {

    fn parse(mut self) -> Result<Chart, ParseError> {

        let version = self.verify()?;
        let section = self.next_section()?;

        println!("Version {}", version);
        println!("Section [{}]", section);

        Ok(Chart::default())
    }
}
