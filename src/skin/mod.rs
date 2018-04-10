//! A module for reading skins.

use opengl_graphics::GlGraphics;
use graphics::draw_state::DrawState;
use graphics::math;
use image;

use std::io;
use std::error;
use std::fmt;
use std::path;

use config;

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
    ImageError(String, image::ImageError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Io(ref s, ref e) => write!(f, "{}: {}", s, e),
            ParseError::Parse(ref s, Some(ref e)) => write!(f, "{}: {}", s, e),
            ParseError::Parse(ref s, None) => write!(f, "{}", s),
            ParseError::UnknownFormat => write!(f, "Unknown skin format"),
            ParseError::InvalidFile => write!(f, "Invalid skin"),
            ParseError::ImageError(ref s, _) => write!(f, "Error reading image {}", s),
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
            ParseError::ImageError(_, _) => "Error reading image",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        use std::ops::Deref;
        match *self {
            ParseError::Io(_, ref e) => Some(e),
            ParseError::Parse(_, Some(ref e)) => Some(e.deref()),
            ParseError::ImageError(_, ref e) => Some(e),
            _ => Some(self),
        }
    }
}

/// Parse from a directory specified by the path.
///
/// For now, the osu parser is assumed.
pub fn from_path<P: AsRef<path::Path>>(path: P, config: &config::Config) -> Result<Box<Skin>, ParseError> {
    osu_skin::from_path(path.as_ref(), &config.default_osu_skin_path)
}

/// A skin. Should be returned by skin parsers.
pub trait Skin {
    fn draw_play_scene(&self,
                       draw_state: &DrawState,
                       transform: math::Matrix2d,
                       gl: &mut GlGraphics,
                       stage_height: f64,
                       keys_down: &[bool],
                       // column index, start pos, end pos
                       notes: &[(usize, f64, Option<f64>)]);
}
