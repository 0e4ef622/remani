//! A module for reading skins.

use graphics::{math, Graphics};
use texture::{CreateTexture, TextureOp};

use std::{error, fmt, io, path::PathBuf};

use crate::{config, judgement::Judgement};

mod osu_skin;

/// The error type from parsing
#[derive(Debug)]
pub enum ParseError {
    /// IO error
    Io(String, io::Error),
    /// Parsing error
    Parse(String, Option<Box<dyn error::Error>>),
    ImageError(String, image::ImageError),
    TextureError {
        path: PathBuf,
        error: String,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ParseError::Io(ref s, ref e) => write!(f, "{}: {}", s, e),
            ParseError::Parse(ref s, Some(ref e)) => write!(f, "{}: {}", s, e),
            ParseError::Parse(ref s, None) => write!(f, "{}", s),
            ParseError::ImageError(ref s, _) => write!(f, "Error reading image {}", s),
            ParseError::TextureError { ref path, ref error } => write!(f, "Error creating texture for {}: {}", path.display(), error),
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
            ParseError::ImageError(_, _) => "Error reading image",
            ParseError::TextureError { .. } => "Error creating texture",
        }
    }
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ParseError::Io(_, ref e) => Some(e),
            ParseError::Parse(_, Some(ref e)) => Some(&**e),
            ParseError::Parse(_, None) => None,
            ParseError::ImageError(_, ref e) => Some(e),
            ParseError::TextureError { .. } => None,
        }
    }
}

/// Parse from a directory specified by the path.
///
/// For now, the osu parser is assumed (TODO).
pub fn from_path<G, F>(
    factory: &mut F,
    skin_entry: &config::SkinEntry,
    config: &config::Config,
) -> Result<Box<dyn GameSkin<G>>, ParseError>
where
    G: Graphics + 'static,
    G::Texture: CreateTexture<F>,
    <G::Texture as TextureOp<F>>::Error: ToString,
{
    match skin_entry {
        config::SkinEntry::Osu(p) =>
            osu_skin::from_path(factory, p, &config.game.default_osu_skin_path),
        config::SkinEntry::O2Jam(_p) => unimplemented!(),
    }
}

/// A skin. Should be returned by skin parsers.
pub trait GameSkin<G: Graphics> {
    fn draw_play_scene(
        &mut self,
        transform: math::Matrix2d,
        graphics: &mut G,
        stage_height: f64,
        keys_down: &[bool; 7],
        // column index, start pos, end pos
        notes: &[(usize, f64, Option<f64>)],
    );
    fn draw_judgement(&mut self, column: usize, judgement: Judgement);
    fn key_down(&mut self, column: usize);
    fn key_up(&mut self, column: usize);
    fn single_note_hit_anim(&mut self, _column: usize) {}
    fn long_note_hit_anim_start(&mut self, column: usize) {
        self.single_note_hit_anim(column);
    }
    fn long_note_hit_anim_stop(&mut self, _column: usize) {}
}
