//! A module for reading charts, or beatmaps.

use std::io;
use std::error;
use std::fmt;
use std::fs::File;
use std::path::Path;

/// A regular note in a chart.
pub struct SimpleNote {
    /// Where in the note it is, in seconds.
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
    /// IO error
    Io(io::Error),
    /// Parsing error
    Parse,
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> ParseError {
        ParseError::Io(e)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Io(ref e) => fmt::Display::fmt(e, f),
            ParseError::Parse => write!(f, "Parse error"),
        }
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Io(ref e) => e.description(),
            ParseError::Parse => "There was a problem parsing the chart",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ParseError::Io(ref e) => Some(e),
            ParseError::Parse => Some(&ParseError::Parse),
        }
    }
}

/// Holds chart data, such as notes, BPM, SV changes, and what not.
#[derive(Default)]
pub struct Chart {
    notes: Vec<Note>,
    timing_points: Vec<TimingPoint>,

    /// Length of the whole song, in seconds
    length: f64,
}

impl Chart {

    /// Parse the chart with the .osu parser
    pub fn from_osu<T: io::BufRead>(input: T) -> Result<Chart, ParseError> {

        for line in input.lines() {
            println!("{}", line?);
        }

        Ok(Chart::default())
    }

    pub fn from_osu_path<T: AsRef<Path>>(path: T) -> Result<Chart, ParseError> {
        let file = File::open(path)?;
        Chart::from_osu(io::BufReader::new(file))
    }

}


