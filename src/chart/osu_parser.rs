//! Osu parser module

use std::io;
use std::io::BufRead;

use chart::{ Chart, ChartParser, ParseError };

/// Verifies that the file headers are correct and returns the file format
/// version
fn verify(line: &str) -> Result<i32, ParseError> {

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

/// Returns string slice containing the section name
fn parse_section(line: &str) -> &str {
    &line[1..line.len()-1]
}

/// Parses a key/value pair separated and returns them in a tuple
fn parse_key_value(line: &str) -> Result<(&str, &str), ParseError> {
    let (a, b) = line.split_at(match line.find(':') {
        Some(n) => n,
        None => return Err(ParseError::Parse(String::from("Malformed key/value pair"), None)),
    });
    Ok((a.trim(), b[1..].trim()))
}

/// Parses .osu charts and returns a `Chart`
#[derive(Default)]
pub struct OsuParser {
    current_section: Option<String>,
}

impl OsuParser {

    fn parse_line(&mut self, line: &str) -> Result<(), ParseError> {
        if line.len() == 0 { return Ok(()); }
        match &line[0..1] {

            "[" => self.current_section = Some(parse_section(line).to_owned()),

            _ => match self.current_section {

                Some(ref s) => match s.as_str() {
                    "General" => {
                        let (k, v) = parse_key_value(line)?;
                        println!("[{}] {} = {}", s, k, v);
                    },
                    _ => (),
                },
                None => return Err(ParseError::InvalidFile),
            },
        }
        Ok(())
    }
}

impl ChartParser for OsuParser {

    fn parse<R: io::BufRead>(mut self, reader: R) -> Result<Chart, ParseError> {

        let read_error = |e| Err(ParseError::Io(String::from("Error reading chart"), e));

        let mut lines = reader.lines();
        let line = match lines.next() {
            Some(r) => match r {
                Ok(s) => s,
                Err(e) => return read_error(e),
            },
            None => return Err(ParseError::InvalidFile),
        };
        println!("Version {}", verify(line.trim())?);

        for line in lines {
            match line {
                Ok(line) => self.parse_line(line.trim())?,
                Err(e) => return read_error(e),
            }
        }

        Ok(Chart::default())
    }
}
