//! A module for reading charts, or beatmaps.

use std::io;
use std::error;
use std::fmt;
use std::ffi;
use std::fs::File;
use std::path::{ Path, PathBuf };

mod osu_parser;

use self::osu_parser::OsuParser;

/// Either a long note or a regular note. The existence of end_time signifies whether this is a long
/// note or not.
#[derive(Debug)]
pub struct Note {

    /// Where the note begins, in seconds.
    pub time: f64,

    /// The column the note is on, with 0 being the first column.
    pub column: usize,

    /// Where the note ends, in seconds. None means it's a regular note, Some means it's a long note.
    pub end_time: Option<f64>,

    // TODO
    // /// The sound to play when the note is hit.
    // pub sound: Rc<something>
}

#[derive(Copy, Clone, Debug)]
pub enum TimingPointValue {
    BPM(f64),
    /// The multipler on the scroll speed
    SV(f64),
}

impl TimingPointValue {

    pub fn inner(&self) -> f64 {
        match *self {
            TimingPointValue::SV(v) => v,
            TimingPointValue::BPM(v) => v,
        }
    }
}

/// Represents either an SV change or a BPM change
#[derive(Debug)]
pub struct TimingPoint {
    /// The offset from the start of the song, in seconds.
    pub offset: f64,
    pub value: TimingPointValue,
}

impl TimingPoint {
    pub fn is_bpm(&self) -> bool {
        match self.value {
            TimingPointValue::BPM(_) => true,
            _ => false,
        }
    }

    pub fn is_sv(&self) -> bool {
        match self.value {
            TimingPointValue::SV(_) => true,
            _ => false,
        }
    }
}

/// The error type from parsing
#[derive(Debug)]
pub enum ParseError {
    /// IO error
    Io(String, io::Error),
    /// Parsing error
    Parse(String, Option<Box<dyn error::Error>>),
    UnknownFormat,
    InvalidFile,
    EOL,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ParseError::Io(ref s, ref e) => write!(f, "{}: {}", s, e),
            ParseError::Parse(ref s, Some(ref e)) => write!(f, "{}: {}", s, e),
            ParseError::Parse(ref s, None) => write!(f, "{}", s),
            ParseError::UnknownFormat => write!(f, "Unknown chart format"),
            ParseError::InvalidFile => write!(f, "Invalid chart"),
            ParseError::EOL => write!(f, "Unexpected end of line"),
        }
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Io(_, _) => "IO error",
            ParseError::Parse(_, _) => "Parse error",
            ParseError::UnknownFormat => "Unknown chart format",
            ParseError::InvalidFile => "Invalid chart",
            ParseError::EOL => "Unexpected end of line",
        }
    }
    fn cause(&self) -> Option<&dyn error::Error> {
        use std::ops::Deref;
        match *self {
            ParseError::Io(_, ref e) => Some(e),
            ParseError::Parse(_, Some(ref e)) => Some(e.deref()),
            _ => Some(self),
        }
    }
}

/// Holds chart data, such as notes, BPM, SV changes, and what not.
#[derive(Default, Debug)]
pub struct Chart {
    pub notes: Vec<Note>,
    pub timing_points: Vec<TimingPoint>,

    /// The bpm for most of the song
    pub primary_bpm: f64,

    /// The creator of the chart
    pub creator: Option<String>,

    /// The song's artist in ASCII
    pub artist: Option<String>,

    /// The song's artist in Unicode
    pub artist_unicode: Option<String>,

    /// The name of the song in ASCII
    pub song_name: Option<String>,

    /// The name of the song in Unicode
    pub song_name_unicode: Option<String>,

    pub difficulty_name: String,

    /// Path to the music audio file, relative to the chart's directory
    pub music_path: PathBuf,
}

impl Chart {

    /// Parse from a file specified by the path.
    ///
    /// The function will choose a parser based on the file extension.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Chart, ParseError> {

        let file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => return Err(ParseError::Io(
                    format!("Error opening {}", path.as_ref().display()), e)),
        };

        match path.as_ref().extension().and_then(ffi::OsStr::to_str) {

            Some("osu") => {
                println!("Using osu parser");
                let parser = OsuParser::default();
                parser.parse(io::BufReader::new(file))
            },

            _ => {
                Err(ParseError::UnknownFormat)
            }
        }
    }
}

/// A chart parser. Should be implemented by chart builders/parsers.
trait ChartParser {

    /// Parse the file
    fn parse<R: io::BufRead>(self, reader: R) -> Result<Chart, ParseError>;
}
