//! Handles configuration of the game

use cpal;
use piston::input;
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, path};
use toml;

mod serde_buffer_size;
mod serde_key_bindings;

/// Holds all the configuration values relevant to the gameplay itself, such as like skin
/// paths or key bindings.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct UnverifiedGameConfig {
    /// Timing offset, in seconds. Applies to visual and timing judgement. Positive means you have
    /// to hit later, and vice versa.
    offset: f64,
    scroll_speed: f64,

    default_osu_skin_path: path::PathBuf,
    current_skin: String,
    current_judge: String,

    skins: BTreeMap<String, SkinEntry>,
    judges: BTreeMap<String, Judge>,

    #[serde(with = "serde_key_bindings")]
    key_bindings: [input::Button; 7],
}

#[derive(Clone, Debug)]
pub struct GameConfig {
    /// Timing offset, in seconds. Applies to visual and timing judgement. Positive means you have
    /// to hit later, and vice versa.
    pub offset: f64,
    pub scroll_speed: f64,

    pub default_osu_skin_path: path::PathBuf,

    /// An index into the `skins` field
    current_skin_index: usize,
    /// An index into the `judges` field
    current_judge_index: usize,

    pub skins: Vec<(String, SkinEntry)>,
    pub judges: Vec<(String, Judge)>,

    #[serde(with = "serde_key_bindings")]
    pub key_bindings: [input::Button; 7],
}

#[derive(Copy, Clone, Debug)]
pub enum GameConfigVerifyError {
    BadCurrentSkin,
    BadCurrentJudge,
}

impl UnverifiedGameConfig {
    fn verify(self) -> Result<GameConfig, GameConfigVerifyError> {

        let skins: Vec<(String, SkinEntry)> = self.skins.into_iter().collect();
        let judges: Vec<(String, Judge)> = self.judges.into_iter().collect();

        let current_skin: String = self.current_skin;
        let current_judge: String = self.current_judge;

        Ok(GameConfig {
            offset: self.offset,
            scroll_speed: self.scroll_speed,
            default_osu_skin_path: self.default_osu_skin_path,

            current_skin_index: skins
                .iter()
                .enumerate()
                .find(|(_, (s, _))| *s == current_skin)
                .ok_or(GameConfigVerifyError::BadCurrentSkin)?
                .0,
            current_judge_index: judges
                .iter()
                .enumerate()
                .find(|(_, (s, _))| *s == current_judge)
                .ok_or(GameConfigVerifyError::BadCurrentJudge)?
                .0,

            skins,
            judges,

            key_bindings: self.key_bindings,
        })
    }
}

impl GameConfig {
    pub fn current_skin(&self) -> &(String, SkinEntry) {
        &self.skins[self.current_skin_index]
    }
    pub fn current_judge(&self) -> &(String, Judge) {
        &self.judges[self.current_judge_index]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Judge {
    pub miss_tolerance: f64,
    pub windows: Vec<[f64; 2]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "path", rename_all = "lowercase")]
pub enum SkinEntry {
    Osu(path::PathBuf),
    O2Jam(path::PathBuf),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeneralConfig {
    pub resolution: [u32; 2],

    #[serde(with = "serde_buffer_size")]
    pub audio_buffer_size: cpal::BufferSize,

    pub chart_path: Vec<ChartPath>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "path", rename_all = "lowercase")]
pub enum ChartPath {
    Osu(path::PathBuf),
    O2Jam(path::PathBuf),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UnverifiedConfig {
    general: GeneralConfig,
    game: UnverifiedGameConfig,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub game: GameConfig,
}


/// Load configuration from a file except that part isn't implemented yet. TODO
pub fn get_config() -> Config {
    // TODO get config path from env var and read/write it
    default_config()
}

/// Create the default configuration
fn default_config() -> Config {
    use piston::input::{keyboard::Key, Button::Keyboard};

    let mut skin_map = BTreeMap::new();
    skin_map.insert("test".into(), SkinEntry::Osu("test/test_skin".into()));

    let mut judge_map = BTreeMap::new();
    judge_map.insert(
        "easy".into(),
        Judge {
            miss_tolerance: 1.0,
            windows: vec![[0.05, -0.05], [0.1, -0.1], [0.2, -0.2]],
        },
    );
    judge_map.insert(
        "hell".into(),
        Judge {
            miss_tolerance: 2.0,
            windows: vec![[0.005, -0.005], [0.008, -0.008], [0.013, -0.013]],
        },
    );

    Config {
        general: GeneralConfig {
            resolution: [1600, 900],
            audio_buffer_size: cpal::BufferSize::Fixed(1024),
            chart_path: vec![], // TODO use directories crate
        },
        game: UnverifiedGameConfig {
            key_bindings: [
                Keyboard(Key::S),
                Keyboard(Key::D),
                Keyboard(Key::F),
                Keyboard(Key::Space),
                Keyboard(Key::J),
                Keyboard(Key::K),
                Keyboard(Key::L),
            ],

            // TODO decide whether to include this in the binary or not
            default_osu_skin_path: path::PathBuf::from("default_osu_skin"),
            current_skin: "test".into(),
            current_judge: "easy".into(),
            skins: skin_map,
            judges: judge_map,
            scroll_speed: 1.3,
            offset: -0.1,
        }.verify().unwrap(),
    }
}
