//! A module for reading charts, or beatmaps.

use std::io;
use std::error;
use std::fmt;
use std::ffi;
use std::fs::File;
use std::path::{ Path, PathBuf };

mod osu_parser;

use self::osu_parser::OsuParser;

/// A regular note in a chart.
#[derive(Debug)]
pub struct SimpleNote {
    /// Where the note is, in seconds.
    pub time: f64
}

/// A long note in a chart.
#[derive(Debug)]
pub struct LongNote {

    /// Where the long note begins, in seconds.
    pub time: f64,

    /// Where the long note ends, in seconds.
    pub end: f64,
}

/// Either a long note or a regular note.
#[derive(Debug)]
pub enum Note {
    Long(LongNote),
    Simple(SimpleNote),
}

/// Represents a change in the BPM of the song.
#[derive(Debug)]
pub struct BPM {
    /// The offset from the start of the song, in seconds.
    pub offset: f64,
    pub bpm: f64,
}

/// Represents a change in the SV of the song.
#[derive(Debug)]
pub struct SV {
    /// The offset from the start of the song, in seconds.
    pub offset: f64,

    /// The SV multiplier.
    pub sv: f64,
}

/// Represents either an SV change or a BPM change
#[derive(Debug)]
pub enum TimingPoint {
    BPM(BPM),
    SV(SV),
}

/// The error type from parsing
#[derive(Debug)]
pub enum ParseError {
    /// IO error
    Io(String, io::Error),
    /// Parsing error
    Parse(String, Option<Box<error::Error>>),
    InvalidChar(char),
    UnknownFormat,
    InvalidFile,
    EOF,
    EOL,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Io(ref s, ref e) => write!(f, "{}: {}", s, e),
            ParseError::Parse(ref s, Some(ref e)) => write!(f, "{}: {}", s, e),
            ParseError::Parse(ref s, None) => write!(f, "{}", s),
            ParseError::InvalidChar(c) => write!(f, "Invalid character `{}'", c),
            ParseError::UnknownFormat => write!(f, "Unknown chart format"),
            ParseError::InvalidFile => write!(f, "Invalid chart"),
            ParseError::EOF => write!(f, "Unexpected EOF"),
            ParseError::EOL => write!(f, "Unexpected end of line"),
        }
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Io(_, _) => "IO error",
            ParseError::Parse(_, _) => "Parse error",
            ParseError::InvalidChar(_) => "Invalid character",
            ParseError::UnknownFormat => "Unknown chart format",
            ParseError::InvalidFile => "Invalid chart",
            ParseError::EOF => "Unexpected EOF",
            ParseError::EOL => "Unexpected end of line",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
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

/// Same as `Chart`, but everything is an `Option`
#[derive(Default)]
struct IncompleteChart {
    notes: Vec<Note>,
    timing_points: Vec<TimingPoint>,
    creator: Option<String>,
    artist: Option<String>,
    artist_unicode: Option<String>,
    song_name: Option<String>,
    song_name_unicode: Option<String>,
    difficulty_name: Option<String>,
    music_path: Option<PathBuf>,
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
