//! Osu parser module

use std::io;
use std::io::BufRead;

use chart::{ Chart, IncompleteChart, ChartParser, ParseError };
use chart::{ SimpleNote, LongNote, Note, TimingPoint };

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

/// Parse a line from the General section
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

/// Parse a line from the TimingPoints section
fn parse_timing_points(line: &str, chart: &mut IncompleteChart, last_tp_index: Option<usize>) -> Result<usize, ParseError> {

    static err_string: &str = "Error parsing timing points";

    let mut last_index = 0;

    let mut offset: Option<f64> = None;
    let mut bpm: Option<f64> = None;

    let mut absolute = true;

    for (index, field) in line.split(',').enumerate() {

        // Keep track of how many fields there were
        last_index = index;

        let n = match field.parse::<f64>() {
            Ok(n) => n,
            Err(e) => return Err(ParseError::Parse(err_string.to_owned(), Some(Box::new(e)))),
        };

        match index {
            0 => offset = Some(n / 1000.0),
            1 => if n.is_sign_positive() {
                bpm = Some(60000.0 / n);
            } else {
                let last_tp = match last_tp_index {
                    Some(e) => &chart.timing_points[e],
                    None => return Err(ParseError::Parse(err_string.to_owned(),
                            Some(Box::new(ParseError::Parse(String::from("Missing root timing point"), None))))),
                };

                bpm = Some(last_tp.bpm * -n / 100.0);
                absolute = false;
            },
            _ => (),
        }
    }
    if last_index != 7 {
        return Err(ParseError::Parse(err_string.to_owned(),
                                     Some(Box::new(ParseError::EOL))));
    }
    println!("Got timing point with offset = {} and bpm = {}", offset.unwrap(), bpm.unwrap());

    let timing_point = TimingPoint { offset: offset.unwrap(), bpm: bpm.unwrap() };
    chart.timing_points.push(timing_point);

    if absolute {
        Ok(chart.timing_points.len() - 1)
    } else {
        Ok(last_tp_index.unwrap())
    }
}

/// Parses .osu charts and returns a `Chart`
#[derive(Default)]
pub struct OsuParser {
    current_section: Option<String>,
    chart: IncompleteChart,
    // tp = timing point
    last_tp_index: Option<usize>,
}

impl OsuParser {

    fn parse_line(&mut self, line: &str) -> Result<(), ParseError> {
        if line.len() == 0 { return Ok(()); }
        match &line[0..1] {

            "[" => self.current_section = Some(parse_section(line).to_owned()),

            _ => match self.current_section {

                Some(ref s) => match s.as_str() {
                    "General" => parse_general(line, &mut self.chart)?,
                    "TimingPoints" => self.last_tp_index = Some(parse_timing_points(line, &mut self.chart, self.last_tp_index)?),
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
