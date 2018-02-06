//! Osu parser module

use std::io;
use std::io::BufRead;

use chart::{ Chart, IncompleteChart, ChartParser, ParseError };

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

fn parse_general(line: &str, chart: &mut IncompleteChart) -> Result<(), ParseError> {
    let (k, v) = line.split_at(match line.find(':') {
        Some(n) => n,
        None => return Err(ParseError::Parse(String::from("Malformed key/value pair"), None)),
    });
    let v = &v[2..];
    println!("[{}] {} = {}", "General", k, v);
    match k {
        "AudioFilename" => chart.music_path = Some(v.into()),
        "Mode" => if v != "3" {
            return Err(ParseError::Parse(
                    String::from("Osu chart is wrong gamemode"), None));
        },
        _ => (),
    }
    Ok(())
}

/// Parses .osu charts and returns a `Chart`
#[derive(Default)]
pub struct OsuParser {
    current_section: Option<String>,
    chart: IncompleteChart,
}

impl OsuParser {

    fn parse_line(&mut self, line: &str) -> Result<(), ParseError> {
        if line.len() == 0 { return Ok(()); }
        match &line[0..1] {

            "[" => self.current_section = Some(parse_section(line).to_owned()),

            _ => match self.current_section {

                Some(ref s) => match s.as_str() {
                    "General" => parse_general(line, &mut self.chart)?,
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

        Ok(Chart {

            music_path: match self.chart.music_path {
                Some(s) => s,
                None => return Err(
                    ParseError::Parse(String::from("Could not find audio file"), None)),
            },

            ..Default::default()
        })
    }
}
