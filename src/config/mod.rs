//! Handles configuration of the game

use piston::input;
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, env, fmt, fs, io, path};

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
    osu_hitsound_enable: bool,

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

    /// Osu mania has two types of sounds: hitsounds and keysounds. Keysounds are sounds that are
    /// supposed to make up part of the music e.g. piano notes. Hitsounds are sounds that aren't
    /// part of the music, but are meant to provide audio feedback, like a drum kick or whistle. I
    /// don't know if there are any maps that put the keysounds where the hitsounds are supposed to
    /// be, so this setting is here.
    pub osu_hitsound_enable: bool,

    pub skins: Vec<(String, SkinEntry)>,
    pub judges: Vec<(String, Judge)>,

    pub key_bindings: [input::Button; 7],
}

#[derive(Copy, Clone, Debug)]
pub enum GameConfigVerifyError {
    BadCurrentSkin,
    BadCurrentJudge,
}

impl UnverifiedGameConfig {
    fn verify(self) -> Result<GameConfig, GameConfigVerifyError> {

        let mut skins: Vec<(String, SkinEntry)> = self.skins.into_iter().collect();
        let mut judges: Vec<(String, Judge)> = self.judges.into_iter().collect();

        skins.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        judges.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let current_skin: String = self.current_skin;
        let current_judge: String = self.current_judge;

        Ok(GameConfig {
            offset: self.offset,
            scroll_speed: self.scroll_speed,
            default_osu_skin_path: self.default_osu_skin_path,

            current_skin_index: skins
                .binary_search_by_key(&&current_skin, |v| &v.0)
                .map_err(|_| GameConfigVerifyError::BadCurrentSkin)?,
            current_judge_index: judges
                .binary_search_by_key(&&current_judge, |v| &v.0)
                .map_err(|_| GameConfigVerifyError::BadCurrentJudge)?,

            osu_hitsound_enable: self.osu_hitsound_enable,

            skins,
            judges,

            key_bindings: self.key_bindings,
        })
    }
}

impl From<GameConfig> for UnverifiedGameConfig {
    fn from(game_config: GameConfig) -> Self {
        UnverifiedGameConfig {
            current_skin: game_config.current_skin().0.clone(),
            current_judge: game_config.current_judge().0.clone(),
            offset: game_config.offset,
            scroll_speed: game_config.scroll_speed,
            default_osu_skin_path: game_config.default_osu_skin_path,
            osu_hitsound_enable: game_config.osu_hitsound_enable,
            skins: game_config.skins.into_iter().collect(),
            judges: game_config.judges.into_iter().collect(),
            key_bindings: game_config.key_bindings,
        }
    }
}

impl GameConfig {
    /// The string is the name of the skin
    pub fn current_skin(&self) -> &(String, SkinEntry) {
        &self.skins[self.current_skin_index]
    }
    /// The string is the name of the judge
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

impl From<Config> for UnverifiedConfig {
    fn from(c: Config) -> Self {
        UnverifiedConfig {
            general: c.general,
            game: c.game.into(),
        }
    }
}

#[derive(Debug)]
pub enum ConfigReadError {
    /// An error in the toml formatting
    Toml(toml::de::Error),
    /// An error somewhere in file IO
    Io(io::Error),
    /// An error in the values of the config
    ConfigError(GameConfigVerifyError),
}

impl From<io::Error> for ConfigReadError {
    fn from(t: io::Error) -> Self {
        ConfigReadError::Io(t)
    }
}

impl From<toml::de::Error> for ConfigReadError {
    fn from(t: toml::de::Error) -> Self {
        ConfigReadError::Toml(t)
    }
}

impl From<GameConfigVerifyError> for ConfigReadError {
    fn from(t: GameConfigVerifyError) -> Self {
        ConfigReadError::ConfigError(t)
    }
}

impl fmt::Display for ConfigReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigReadError::Toml(e) => write!(f, "Formatting error: {}", e),
            ConfigReadError::Io(e) => write!(f, "IO error: {}", e),
            ConfigReadError::ConfigError(e) => write!(f, "Config error: {:?}", e),
        }
    }
}

/// Load configuration from a file except that part isn't implemented yet.
fn read_config_from_path<P: AsRef<path::Path>>(config_file_path: P) -> Result<Config, ConfigReadError> {
    let file_data = fs::read(config_file_path)?;
    let config = toml::from_slice::<UnverifiedConfig>(&file_data)?;

    Ok(Config {
        general: config.general,
        game: config.game.verify()?,
    })
}

pub fn get_config<P: AsRef<path::Path>>(config_file_path: P) -> Config {
    match read_config_from_path(config_file_path.as_ref()) {
        Ok(c) => c,
        Err(e) => {
            remani_warn!("Error reading from {}: {}", config_file_path.as_ref().display(), e);
            remani_warn!("Using default config");
            default_config()
        }
    }
}

#[derive(Debug)]
pub enum ConfigWriteError {
    Io(io::Error),
    Toml(toml::ser::Error),
}

impl fmt::Display for ConfigWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ConfigWriteError::Io(e) => write!(f, "{}", e),
            ConfigWriteError::Toml(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ConfigWriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigWriteError::Io(e) => Some(e),
            ConfigWriteError::Toml(e) => Some(e),
        }
    }
}

impl From<toml::ser::Error> for ConfigWriteError {
    fn from(t: toml::ser::Error) -> Self {
        ConfigWriteError::Toml(t)
    }
}

impl From<io::Error> for ConfigWriteError {
    fn from(t: io::Error) -> Self {
        ConfigWriteError::Io(t)
    }
}

pub fn write_config_to_path<P: AsRef<path::Path>>(config: Config, path: P) -> Result<(), ConfigWriteError> {
    use std::io::Write;
    let path = path.as_ref();
    match path.parent() {
        Some(p) => std::fs::create_dir_all(p)?,
        None => (),
    }
    let mut file = fs::File::create(path)?;
    file.write(toml::ser::to_string(&UnverifiedConfig::from(config))?.as_bytes())?;
    Ok(())
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
            resolution: [800, 600],
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
            default_osu_skin_path: path::PathBuf::from("rsc/default_osu_skin"),
            current_skin: "test".into(),
            current_judge: "easy".into(),
            osu_hitsound_enable: false,
            skins: skin_map,
            judges: judge_map,
            scroll_speed: 1.7,
            offset: -0.1,
        }.verify().unwrap(),
    }
}

pub fn config_path() -> path::PathBuf {
    env::var_os("REMANI_CONF")
        .map(|s| s.into())
        .unwrap_or(directories::ProjectDirs::from("", "0e4ef622", "Remani")
            .unwrap()
            .config_dir()
            .join("config.toml")
        )
}
