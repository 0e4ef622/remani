//! A module for reading skins.

use graphics::math;
use graphics::Graphics;
use image;
use texture::CreateTexture;

use std::io;
use std::error;
use std::fmt;
use std::path;

use crate::config;
use crate::judgement::Judgement;

mod osu_skin;

/// The error type from parsing
#[derive(Debug)]
pub enum ParseError {
    /// IO error
    Io(String, io::Error),
    /// Parsing error
    Parse(String, Option<Box<dyn error::Error>>),
    InvalidFile,
    ImageError(String, image::ImageError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Io(ref s, ref e) => write!(f, "{}: {}", s, e),
            ParseError::Parse(ref s, Some(ref e)) => write!(f, "{}: {}", s, e),
            ParseError::Parse(ref s, None) => write!(f, "{}", s),
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
            ParseError::InvalidFile => "Invalid skin",
            ParseError::ImageError(_, _) => "Error reading image",
        }
    }
    fn cause(&self) -> Option<&dyn error::Error> {
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
/// For now, the osu parser is assumed (TODO).
pub fn from_path<P, G, F>(factory: &mut F, path: P, config: &config::Config) -> Result<Box<dyn Skin<G>>, ParseError>
where G: Graphics + 'static, G::Texture: CreateTexture<F>, <G::Texture as CreateTexture<F>>::Error: ToString, P: AsRef<path::Path>
{
    osu_skin::from_path(factory, path.as_ref(), &config.default_osu_skin_path)
}

/// A skin. Should be returned by skin parsers.
pub trait Skin<G: Graphics> {
    fn draw_play_scene(&mut self,
                       transform: math::Matrix2d,
                       graphics: &mut G,
                       stage_height: f64,
                       keys_down: &[bool],
                       // column index, start pos, end pos
                       notes: &[(usize, f64, Option<f64>)]);
    fn draw_judgement(&mut self, column: usize, judgement: Judgement);
    fn key_down(&mut self, column: usize);
    fn key_up(&mut self, column: usize);
    fn single_note_hit_anim(&mut self, _column: usize) { }
    fn long_note_hit_anim_start(&mut self, column: usize) {
        self.single_note_hit_anim(column);
    }
    fn long_note_hit_anim_stop(&mut self, _column: usize) {
    }
}
