//! A module for reading skins.

extern crate opengl_graphics;

use opengl_graphics::Texture;

use std::io;
use std::error;
use std::fmt;
use std::path;

mod osu_parser;

use self::osu_parser::OsuParser;

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

/// Holds skin data, such as note images and what not.
#[derive(Default)]
pub struct Skin {
    pub miss: Vec<Texture>,
    pub hit50: Vec<Texture>,
    pub hit100: Vec<Texture>,
    pub hit300: Vec<Texture>,
    pub hit300g: Vec<Texture>,

    pub key1: Vec<Texture>,
    pub key2: Vec<Texture>,
    pub key3: Vec<Texture>,
    pub key4: Vec<Texture>,

    pub key1D: Vec<Texture>,
    pub key2D: Vec<Texture>,
    pub key3D: Vec<Texture>,
    pub key4D: Vec<Texture>,

    pub note1: Vec<Texture>,
    pub note2: Vec<Texture>,
    pub note3: Vec<Texture>,

    pub note1H: Vec<Texture>,
    pub note2H: Vec<Texture>,
    pub note3H: Vec<Texture>,

    pub note1L: Vec<Texture>,
    pub note2L: Vec<Texture>,
    pub note3L: Vec<Texture>,
}

impl Skin {

    /// Parse from a directory specified by the path.
    ///
    /// For now, the osu parser is assumed.
    pub fn from_path<P: AsRef<path::Path>>(path: P) -> Result<Skin, ParseError> {

        let parser = OsuParser::new(path::PathBuf::new().join(&path));
        parser.parse()
    }
}

/// A skin parser. Should be implemented by skin builders/parsers.
trait SkinParser {

    /// Parse the directory
    fn parse(self) -> Result<Skin, ParseError>;
}
