//! A module for reading skins.

use opengl_graphics::GlGraphics;
use graphics::draw_state::DrawState;
use graphics::math;

use std::io;
use std::error;
use std::fmt;
use std::path;

mod osu_skin;

/// The error type from parsing
#[derive(Debug)]
pub enum ParseError {
    /// IO error
    Io(String, io::Error),
    /// Parsing error
    Parse(String, Option<Box<error::Error>>),
    UnknownFormat,
    InvalidFile,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Io(ref s, ref e) => write!(f, "{}: {}", s, e),
            ParseError::Parse(ref s, Some(ref e)) => write!(f, "{}: {}", s, e),
            ParseError::Parse(ref s, None) => write!(f, "{}", s),
            ParseError::UnknownFormat => write!(f, "Unknown skin format"),
            ParseError::InvalidFile => write!(f, "Invalid skin"),
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> Self {
        ParseError::Io(String::new(), error)
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Io(_, _) => "IO error",
            ParseError::Parse(_, _) => "Parse error",
            ParseError::UnknownFormat => "Unknown skin format",
            ParseError::InvalidFile => "Invalid skin",
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

/// Parse from a directory specified by the path.
///
/// For now, the osu parser is assumed.
pub fn from_path<P: AsRef<path::Path>>(path: P) -> Result<Box<Skin>, ParseError> {
    Ok(osu_skin::from_path(path.as_ref())?)
}

/// A skin. Should be returned by skin parsers.
pub trait Skin {
    fn draw_note(&self, draw_state: &DrawState, transform: math::Matrix2d, gl: &mut GlGraphics, y_pos: f64, column_index: usize);
    fn draw_track(&self, draw_state: &DrawState, transform: math::Matrix2d, gl: &mut GlGraphics);
}
