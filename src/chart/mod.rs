//! A module for reading charts, or beatmaps.

use cpal;

use crate::audio;

use std::{error, fmt, io, rc::Rc};

pub mod osu;
pub mod ojn;

// TODO temporary for testing
pub use self::ojn::dump_data as ojn_dump;
pub use self::ojn::ojm_dump;

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
    // pub sound: Option<Rc<audio::EffectStream>>
}

#[derive(Copy, Clone, Debug)]
pub enum TimingPointValue {
    BPM(f64),
    /// The multipler on the scroll speed, currently only used by osu
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
            ParseError::Io(..) => "IO error",
            ParseError::Parse(..) => "Parse error",
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
            _ => None,
        }
    }
}

/// Holds chart data, such as notes, BPM, SV changes, and what not.
pub trait Chart {
    fn notes(&self) -> &[Note];
    fn timing_points(&self) -> &[TimingPoint];

    /// The bpm for most of the song
    fn primary_bpm(&self) -> f64;

    /// The creator of the chart
    fn creator(&self) -> Option<&str> { None }

    /// The song's artist in ASCII
    fn artist(&self) -> Option<&str> { None }

    /// The song's artist in Unicode
    fn artist_unicode(&self) -> Option<&str> { None }

    /// The name of the song in ASCII
    fn song_name(&self) -> Option<&str> { None }

    /// The name of the song in Unicode
    fn song_name_unicode(&self) -> Option<&str> { None }

    fn difficulty_name(&self) -> &str;

    /// A mutable reference to the music wrapped inside an `Option` so that you can call `take` on
    /// it.
    fn music(&mut self, format: &cpal::Format) -> Result<audio::MusicStream, audio::AudioLoadError>;
}
