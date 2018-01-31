//! A module for reading charts, or beatmaps.

use std::io;
use std::error;
use std::fmt;
use std::ffi;
use std::fs::File;
use std::path::Path;

mod osu_parser;

use self::osu_parser::OsuParser;

/// A regular note in a chart.
pub struct SimpleNote {
    /// Where the note is, in seconds.
    pub time: f64
}

/// A long note in a chart.
pub struct LongNote {

    /// Where the long note begins, in seconds.
    pub time: f64,

    /// Where the long note ends, in seconds.
    pub end: f64,
}

/// Either a long note or a regular note.
pub enum Note {
    Long(LongNote),
    Simple(SimpleNote),
}

/// Represents a change in the timing of the song.
pub struct TimingPoint {
    pub offset: f64,
    pub bpm: f64,
}

/// The error type from parsing
#[derive(Debug)]
pub enum ParseError {
    Parse(String),
    UnknownFormat,
    InvalidFile,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Parse(ref s) => write!(f, "Parse error: {}", s),
            ParseError::UnknownFormat => write!(f, "Unknown chart format"),
            ParseError::InvalidFile => write!(f, "Invalid chart"),
        }
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Parse(_) => "Parse error",
            ParseError::UnknownFormat => "Unknown chart format",
            ParseError::InvalidFile => "Invalid chart",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        Some(self)
    }
}

/// Holds chart data, such as notes, BPM, SV changes, and what not.
#[derive(Default)]
pub struct Chart {
    pub notes: Vec<Note>,
    pub timing_points: Vec<TimingPoint>,

    /// Length of the whole song, in seconds
    pub length: f64,
}

impl Chart {

    /// Parse from a file specified by the path.
    ///
    /// The function will choose a parser based on the file extension.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Chart, Box<error::Error>> {

        let file = File::open(&path)?;

        match path.as_ref().extension().and_then(ffi::OsStr::to_str) {

            Some("osu") => {
                println!("Using osu parser");
                let parser = OsuParser::new(file);
                parser.parse()
            },

            _ => {
                Err(Box::new(ParseError::UnknownFormat))
            }
        }
    }
}

/// A chart parser. Should be implemented by chart builders/parsers.
trait ChartParser {

    /// Parse the file
    fn parse(self) -> Result<Chart, Box<error::Error>>;
}
