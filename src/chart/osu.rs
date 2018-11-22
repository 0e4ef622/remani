//! Osu chart parser module

use either::Either;

use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf}
};

use crate::{
    config::{self, Config},
    chart::{
        self,
        audio,
        AutoplaySound,
        Chart,
        Note,
        ParseError,
    },
};

/// Convert Err values to ParseError
macro_rules! cvt_err {
    ($s:expr, $e:expr) => {
        $e.or_else(|e| Err(ParseError::Parse($s.to_owned(), Some(Box::new(e)))))
    };
}

/// Verifies that the file headers are correct and returns the file format version
fn verify(line: &str) -> Result<i32, ParseError> {
    if !line.starts_with("osu file format v") {
        return Err(ParseError::InvalidFile);
    }

    cvt_err!("Error parsing file version", line[17..].parse::<i32>())
}

/// Returns string slice containing the section name
fn parse_section(line: &str) -> &str {
    &line[1..line.len() - 1]
}

/// Parse a line from the General section and add the info to the chart passed in
fn parse_general(line: &str, chart: &mut IncompleteChart) -> Result<(), ParseError> {
    let (k, v) = line.split_at(match line.find(':') {
        Some(n) => n,
        None => {
            return Err(ParseError::Parse(
                String::from("Error parsing General section: Malformed key/value pair"),
                None,
            ))
        }
    });
    let v = &v[2..];

    match k {
        "AudioFilename" => chart.music_path = Some(v.into()),
        "Mode" => if v != "3" {
            return Err(ParseError::Parse(
                String::from("Osu chart is wrong gamemode"),
                None,
            ));
        },
        _ => (),
    }
    Ok(())
}

/// Parse a line from the Metadata section and add the info to the chart passed in
fn parse_metadata(line: &str, chart: &mut IncompleteChart) -> Result<(), ParseError> {
    let (k, v) = line.split_at(match line.find(':') {
        Some(n) => n,
        None => {
            return Err(ParseError::Parse(
                String::from("Malformed key/value pair"),
                None,
            ))
        }
    });
    let v = &v[1..];

    match k {
        "Title" => chart.song_name = Some(v.into()),
        "TitleUnicode" => chart.song_name_unicode = Some(v.into()),
        "Artist" => chart.artist = Some(v.into()),
        "ArtistUnicode" => chart.artist_unicode = Some(v.into()),
        "Creator" => chart.creator = Some(v.into()),
        "Version" => chart.difficulty_name = Some(v.into()),
        _ => (),
    }
    Ok(())
}

/// Parse a line from the Difficulty section. This just checks to see if the chart is a 7k chart.
fn parse_difficulty(line: &str) -> Result<(), ParseError> {
    let (k, v) = line.split_at(match line.find(':') {
        Some(n) => n,
        None => {
            return Err(ParseError::Parse(
                String::from("Malformed key/value pair"),
                None,
            ))
        }
    });
    let v = &v[1..];

    if k == "CircleSize" && v != "7" {
        Err(ParseError::Parse(
            String::from("This chart is not 7 key"),
            None,
        ))
    } else {
        Ok(())
    }
}

/// Parse a line from the TimingPoints section and add the timing point to the chart passed in
fn parse_timing_point(line: &str, chart: &mut IncompleteChart) -> Result<(), ParseError> {
    static ERR_STRING: &str = "Error parsing timing point";

    let mut last_index = 0;

    let mut offset: Option<f64> = None;
    let mut bpm: Option<f64> = None;
    let mut sv: Option<f64> = None;
    let mut sample_set: Option<SampleSet> = None;
    let mut sample_index: Option<u32> = None;
    let mut volume: Option<u8> = None;

    let mut inherited = false;

    for (index, field) in line.split(',').enumerate().take(8) {
        // Keep track of how many fields there were
        last_index = index;

        match index {
            // offset
            0 => offset = Some(cvt_err!(ERR_STRING, field.parse::<f64>())? / 1000.0),

            // ms per beat or sv
            1 => {
                let n = cvt_err!(ERR_STRING, field.parse::<f64>())?;
                if n.is_sign_positive() {
                    bpm = Some(60000.0 / n);
                } else {
                    sv = Some(100.0 / -n);
                    inherited = true;
                }
            }

            // meter, not important
            2 => (),

            // sample set
            3 => {
                let n = cvt_err!(ERR_STRING, field.parse::<u8>())?;
                sample_set = Some(match n {
                    0 => SampleSet::Auto,
                    1 => SampleSet::Normal,
                    2 => SampleSet::Soft,
                    3 => SampleSet::Drum,
                    x => {
                        remani_warn!("Unknown sample set {}", x);
                        SampleSet::Auto
                    }
                });
            }

            // sample index
            4 => {
                sample_index = Some(cvt_err!(ERR_STRING, field.parse::<u32>())?);
            }

            // volume
            5 => {
                volume = Some(cvt_err!(ERR_STRING, field.parse::<u8>())?);
            }

            // inherited, we're gonna ignore since we determined this when looking at the ms / beat
            6 => (),
            // kiai mode, not important
            7 => (),
            _ => unreachable!(),
        }
    }
    if last_index < 7 {
        return Err(ParseError::Parse(
            ERR_STRING.to_owned(),
            Some(Box::new(ParseError::EOL)),
        ));
    }

    let timing_point_value = if inherited {
        chart::TimingPointValue::SV(sv.unwrap())
    } else {
        chart::TimingPointValue::BPM(bpm.unwrap())
    };
    chart.timing_points.push(OsuTimingPoint {
        offset: offset.unwrap(),
        value: timing_point_value,
        sample_set: sample_set.unwrap(),
        sample_index: sample_index.unwrap(),
        volume: volume.unwrap(),
    });
    Ok(())
}

/// Parse a line from the HitObjects section and add the hit object to the chart passed in
fn parse_hit_object(line: &str, chart: &mut IncompleteChart) -> Result<(), ParseError> {
    let mut last_index = 0;
    const ERR_STRING: &str = "Error parsing hit object";

    let mut ln = false;
    let mut hit_obj = HitObject::default();
    for (index, field) in line.split(',').enumerate().take(6) {
        // Keep track of how many fields there were
        last_index = index;

        match index {
            // x
            0 => {
                // calculate column

                let n = cvt_err!(ERR_STRING, field.parse::<f64>())?;
                const CW: f64 = 512.0 / 7.0;
                let mut c = (n / CW).floor();
                if c < 0.0 {
                    c = 0.0;
                } else if c > 7.0 {
                    c = 7.0;
                }
                hit_obj.column = c as usize;
            }
            // y, irrelevant
            1 => (),
            // time
            2 => hit_obj.time = cvt_err!(ERR_STRING, field.parse::<f64>())? / 1000.0,
            // type
            3 => ln = cvt_err!(ERR_STRING, field.parse::<u8>())? & 128 == 128,
            // hitsound
            4 => {
                let n = cvt_err!(ERR_STRING, field.parse::<u8>())?;

                // constructs a hitsound with some default values
                macro_rules! dflt_hit_snd {
                    ($e:expr) => {
                        HitSound {
                            volume: 100,
                            source: HitSoundSource::SampleSet(SampleHitSound {
                                set: SampleSet::Auto,
                                sound: $e,
                                index: 0,
                            }),
                        }
                    };
                }
                hit_obj
                    .sounds
                    .push(dflt_hit_snd!(SampleHitSoundSound::Normal));
                if n & 2 == 2 {
                    hit_obj
                        .sounds
                        .push(dflt_hit_snd!(SampleHitSoundSound::Whistle));
                }
                if n & 4 == 4 {
                    hit_obj
                        .sounds
                        .push(dflt_hit_snd!(SampleHitSoundSound::Finish));
                }
                if n & 8 == 8 {
                    hit_obj
                        .sounds
                        .push(dflt_hit_snd!(SampleHitSoundSound::Clap));
                }
            }
            // endtime/extras
            5 => {
                let mut extras = field.split(':');
                if ln {
                    hit_obj.end_time = Some(match extras.next() {
                        Some(s) => cvt_err!(ERR_STRING, s.parse::<f64>())? / 1000.0,
                        None => {
                            return Err(ParseError::Parse(
                                ERR_STRING.to_owned(),
                                Some(Box::new(ParseError::EOL)),
                            ))
                        }
                    });
                }
                let mut volume = 100;
                for (i, v) in extras.enumerate().take(5) {
                    match i {
                        // sample set
                        0 => {
                            let hs_iter = hit_obj.sounds.iter_mut();
                            match cvt_err!(ERR_STRING, v.parse::<u8>())? {
                                0 => (),
                                1 => hs_iter.for_each(|s| {
                                    if let HitSoundSource::SampleSet(ref mut shs) = s.source {
                                        shs.set = SampleSet::Normal
                                    }
                                }),
                                2 => hs_iter.for_each(|s| {
                                    if let HitSoundSource::SampleSet(ref mut shs) = s.source {
                                        shs.set = SampleSet::Soft
                                    }
                                }),
                                3 => hs_iter.for_each(|s| {
                                    if let HitSoundSource::SampleSet(ref mut shs) = s.source {
                                        shs.set = SampleSet::Drum
                                    }
                                }),
                                _ => (),
                            }
                        }
                        // addition set
                        1 => {
                            let hs_iter = hit_obj.sounds.iter_mut().filter(|s| {
                                if let HitSoundSource::SampleSet(ref shs) = s.source {
                                    shs.sound != SampleHitSoundSound::Normal
                                } else {
                                    false
                                }
                            });
                            match cvt_err!(ERR_STRING, v.parse::<u8>())? {
                                0 => (),
                                1 => hs_iter.for_each(|s| {
                                    if let HitSoundSource::SampleSet(ref mut shs) = s.source {
                                        shs.set = SampleSet::Normal
                                    }
                                }),
                                2 => hs_iter.for_each(|s| {
                                    if let HitSoundSource::SampleSet(ref mut shs) = s.source {
                                        shs.set = SampleSet::Soft
                                    }
                                }),
                                3 => hs_iter.for_each(|s| {
                                    if let HitSoundSource::SampleSet(ref mut shs) = s.source {
                                        shs.set = SampleSet::Drum
                                    }
                                }),
                                _ => (),
                            }
                        }
                        // custom index
                        2 => match cvt_err!(ERR_STRING, v.parse::<u32>())? {
                            0 => (),
                            n => {
                                hit_obj.sounds.iter_mut().for_each(|s| {
                                    if let HitSoundSource::SampleSet(ref mut shs) = s.source {
                                        shs.index = n
                                    }
                                });
                            }
                        },
                        // volume
                        3 => {
                            let n = cvt_err!(ERR_STRING, v.parse::<u8>())?;
                            if n != 0 {
                                hit_obj.sounds.iter_mut().for_each(|s| s.volume = n);
                                volume = n;
                            }
                        }
                        // hitsound from file
                        4 => if !v.is_empty() {
                            hit_obj.sounds.push(HitSound {
                                volume,
                                source: HitSoundSource::File(PathBuf::from(v)),
                            });
                        },
                        _ => unreachable!(),
                    }
                }
            }
            _ => unreachable!(),
        }
    }
    if last_index < 5 {
        return Err(ParseError::Parse(
            ERR_STRING.to_owned(),
            Some(Box::new(ParseError::EOL)),
        ));
    }

    chart.hit_objects.push(hit_obj);
    Ok(())
}

/// Represents a hit object. This will get converted into a Note once the file is parsed and we can
/// get the audio samples for the hit sound.
#[derive(Default, Debug)]
struct HitObject {
    /// Where the note begins, in seconds.
    time: f64,
    column: usize,
    end_time: Option<f64>,
    sounds: Vec<HitSound>,
}

impl HitObject {
    fn into_note(mut self, sound_cache: &mut HashMap<Vec<HitSound>, usize>, timing_points: &[OsuTimingPoint]) -> Note {
        for sound in &mut self.sounds {
            sound.resolve_tp_inherit(self.time, timing_points);
        }
        let len = sound_cache.len();
        Note {
            time: self.time,
            column: self.column,
            end_time: self.end_time,
            sound_index: Some(*sound_cache.entry(self.sounds).or_insert(len)),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct HitSound {
    /// The audio source of the sample.
    source: HitSoundSource,

    /// The volume of the hit sound, from 0 to 100
    volume: u8,
}

impl HitSound {
    fn load_sound(self,
        chart: &OsuChart,
        format: &cpal::Format,
        config: &Config,
        cache: &mut HashMap<PathBuf, audio::EffectStream>
    ) -> Result<audio::EffectStream, (PathBuf, audio::AudioLoadError)> {
        match self.source {
            HitSoundSource::File(path) => {
                // the extension SHOULD be .wav, if it's not I don't know what to say...
                let mut path = chart.chart_path.join(&path);
                if let Some(effect_stream) = cache.get(&path) {
                    return Ok(effect_stream.clone());
                }
                path.set_extension("ogg"); // this is so stupid the file could say sound.wav but I have to check for sound.ogg as well
                if let Some(effect_stream) = cache.get(&path) {
                    return Ok(effect_stream.clone());
                }
                path.set_extension("wav");
                let effect_stream: audio::EffectStream = match audio::music_from_path(&path, format) {
                    Ok(s) => s.into(),
                    Err(_) => {
                        path.set_extension("ogg");
                        match audio::music_from_path(&path, format) {
                            Ok(s) => s.into(),
                            Err(e) => return Err((path, e)),
                        }
                    }
                };
                cache.insert(path, effect_stream.clone());
                Ok(effect_stream.with_volume(f32::from(self.volume) / 100.0))
            }
            HitSoundSource::SampleSet(shs) if config.game.osu_hitsound_enable => {
                let mut the_path = None;
                for path in shs.possible_paths(config, chart) {
                    if let Some(effect_stream) = cache.get(&path) {
                        return Ok(effect_stream.clone());
                    }
                    if path.is_file() {
                        the_path = Some(path);
                        break;
                    }
                }
                if let Some(path) = the_path {
                    let effect_stream: audio::EffectStream = match audio::music_from_path(&path, format) {
                        Ok(s) => s.into(),
                        Err(e) => return Err((path, e)),
                    };
                    cache.insert(path, effect_stream.clone());
                    Ok(effect_stream.with_volume(f32::from(self.volume) / 100.0))
                } else {
                    remani_warn!("Could not find hitsound: {:?}", self);
                    Ok(audio::EffectStream::empty())
                }
            }
            HitSoundSource::SampleSet(_) => Ok(audio::EffectStream::empty()),
        }
    }
    /// Figures out which timing point this hit sound inherits from, if any, and sets itself
    /// accordingly.
    fn resolve_tp_inherit(&mut self, time: f64, timing_points: &[OsuTimingPoint]) {
        match &mut self.source {
            HitSoundSource::SampleSet(sample) => {
                let mut tp = None;
                if sample.index == 0 {
                    tp = find_tp_inherit_from(time, timing_points);
                    sample.index = tp.map(|t| t.sample_index).unwrap_or(1);
                }
                if sample.set == SampleSet::Auto {
                    tp = tp.or_else(|| find_tp_inherit_from(time, timing_points));
                    sample.set = tp.map(|t| t.sample_set).unwrap_or(SampleSet::Normal);
                }
            }
            HitSoundSource::File(_) => (),
        }
    }
}

/// Find the timing point to inherit from
fn find_tp_inherit_from(time: f64, timing_points: &[OsuTimingPoint]) -> Option<&OsuTimingPoint> {
    if let Some(tp) = timing_points.first() {
        if time < tp.offset {
            Some(tp)
        } else {
            let mut tp_index = 0;
            for (i, timing_point) in timing_points.iter().enumerate() {
                if timing_point.offset > time {
                    break;
                } else {
                    tp_index = i;
                }
            }
            Some(&timing_points[tp_index])
        }
    } else {
        None
    }
}

/// Where to get the audio source of the hit sound
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum HitSoundSource {
    SampleSet(SampleHitSound),
    File(PathBuf),
}

/// A hit sound that comes from a sample set
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
struct SampleHitSound {
    set: SampleSet,
    sound: SampleHitSoundSound,
    index: u32,
}

/// A sample set
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
enum SampleSet {
    Auto,
    Normal,
    Soft,
    Drum,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
enum SampleHitSoundSound {
    Normal,
    Whistle,
    Finish,
    Clap,
}

impl SampleHitSound {
    fn possible_paths(self, config: &Config, chart: &OsuChart) -> impl Iterator<Item = PathBuf> {
        let sample_set = match self.set {
            SampleSet::Auto => "", // TODO inherit from timing point
            SampleSet::Normal => "normal",
            SampleSet::Soft => "soft",
            SampleSet::Drum => "drum",
        };
        let sound = match self.sound {
            SampleHitSoundSound::Normal => "normal",
            SampleHitSoundSound::Whistle => "whistle",
            SampleHitSoundSound::Finish => "finish",
            SampleHitSoundSound::Clap => "clap",
        };
        let index = match self.index {
            0 => String::new(), // TODO special things
            1 => String::new(),
            n => n.to_string(),
        };

        // TODO decide whether to defer the following two format! calls
        let filename_with_index_wav = format!("{}-hit{}{}.wav", sample_set, sound, index);
        let filename_without_index_wav = format!("{}-hit{}.wav", sample_set, sound);
        let path1 = chart.chart_path.join(&filename_with_index_wav);
        let path2 = match &config.game.current_skin().1 {
            config::SkinEntry::Osu(path) => Some(path.join(&filename_without_index_wav)),
            config::SkinEntry::O2Jam(_path) => None,
        };
        let path3 = config.game.default_osu_skin_path.join(&filename_without_index_wav);

        macro_rules! iter {
            ($item:expr) => (std::iter::once($item));
            ($item:expr, $($rest:tt)*) => (std::iter::once($item).chain(iter!($($rest)*)));
        }

        if let Some(p2) = path2 {
            Either::Left(iter![path1, p2, path3])
        } else {
            Either::Right(iter![path1, path3])
        }
    }
}

fn parse_events(line: &str, chart: &mut IncompleteChart) -> Result<(), ParseError> {
    const ERR_STRING: &str = "Error parsing storyboard sound";
    let mut items = line.split(',');
    if items.next() != Some("Sample") {
        return Ok(()); // we don't care about any other lines
    }
    let time = match items.next() {
        Some(s) => cvt_err!(ERR_STRING, s.parse::<f64>())? / 1000.0,
        None => return Err(ParseError::Parse(
            ERR_STRING.to_owned(),
            Some(Box::new(ParseError::EOL)))),
    };
    match items.next() { // layer_num, we don't care about this
        Some(_) => (),
        None => return Err(ParseError::Parse(
            ERR_STRING.to_owned(),
            Some(Box::new(ParseError::EOL)))),
    };
    let volume = match items.last() {
        Some(s) => cvt_err!(ERR_STRING, s.parse::<f32>())? / 100.0,
        None => return Err(ParseError::Parse(
            format!("{}: malformed volume", ERR_STRING),
            None,
        )),
    };
    let mut quotes = line.split('"');
    match quotes.next() {
        Some(_) => (),
        None => return Err(ParseError::Parse(
            format!("{}: malformed line", ERR_STRING),
            None,
        )),
    }
    let sound_file = match quotes.next() {
        Some(s) => s,
        None => return Err(ParseError::Parse(
            format!("{}: malformed line", ERR_STRING),
            None,
        )),
    };
    chart.autoplay_sounds.push(OsuAutoplaySound {
        time,
        volume,
        sound_file: sound_file.into(),
    });
    Ok(())
}

struct OsuAutoplaySound {
    time: f64,
    volume: f32,
    sound_file: PathBuf,
}

impl OsuAutoplaySound {
    fn into_autoplay_sound(self, start_index: usize, cache: &mut HashMap<PathBuf, usize>) -> AutoplaySound {
        let cache_len = cache.len();
        AutoplaySound {
            time: self.time,
            volume: self.volume,
            sound_index: *cache.entry(self.sound_file).or_insert(start_index + cache_len),
        }
    }
}

/// Used during parsing, gets finalized into a Chart once all the values are obtained.
#[derive(Default)]
struct IncompleteChart {
    hit_objects: Vec<HitObject>,
    timing_points: Vec<OsuTimingPoint>,
    autoplay_sounds: Vec<OsuAutoplaySound>,
    creator: Option<String>,
    artist: Option<String>,
    artist_unicode: Option<String>,
    song_name: Option<String>,
    song_name_unicode: Option<String>,
    difficulty_name: Option<String>,
    music_path: Option<PathBuf>,
}

enum MaybeLoadedSounds {
    NotLoaded(Vec<Vec<HitSound>>),
    Loaded(Vec<audio::EffectStream>),
}

impl MaybeLoadedSounds {
    fn len(&self) -> usize {
        match self {
            MaybeLoadedSounds::NotLoaded(v) => v.len(),
            MaybeLoadedSounds::Loaded(v) => v.len(),
        }
    }
}

enum MaybeLoadedAutoplaySounds {
    NotLoaded(Vec<PathBuf>),
    Loaded(Vec<audio::EffectStream>),
}

/// See [`Chart`]
///
/// [`Chart`]: ../trait.Chart.html
struct OsuChart {
    notes: Vec<Note>,
    timing_points: Vec<chart::TimingPoint>,
    autoplay_sounds: Vec<AutoplaySound>,
    primary_bpm: f64,
    creator: Option<String>,
    artist: Option<String>,
    artist_unicode: Option<String>,
    song_name: Option<String>,
    song_name_unicode: Option<String>,
    difficulty_name: String,
    music_path: PathBuf,
    chart_path: PathBuf,
    sounds: MaybeLoadedSounds,
    autoplay_sound_files: MaybeLoadedAutoplaySounds,
}

impl Chart for OsuChart {
    fn notes(&self) -> &[Note] {
        &self.notes
    }
    fn timing_points(&self) -> &[chart::TimingPoint] {
        &self.timing_points
    }
    fn primary_bpm(&self) -> f64 {
        self.primary_bpm
    }
    fn creator(&self) -> Option<&str> {
        self.creator.as_ref().map(|s| &**s)
    }
    fn artist(&self) -> Option<&str> {
        self.artist.as_ref().map(|s| &**s)
    }
    fn artist_unicode(&self) -> Option<&str> {
        self.artist_unicode.as_ref().map(|s| &**s)
    }
    fn song_name(&self) -> Option<&str> {
        self.song_name.as_ref().map(|s| &**s)
    }
    fn song_name_unicode(&self) -> Option<&str> {
        self.song_name_unicode.as_ref().map(|s| &**s)
    }
    fn difficulty_name(&self) -> &str {
        &self.difficulty_name
    }
    fn autoplay_sounds(&self) -> &[AutoplaySound] {
        &self.autoplay_sounds
    }
    fn music(&mut self, format: &cpal::Format) -> Result<audio::MusicStream, audio::AudioLoadError> {
        audio::music_from_path(&self.chart_path.join(&self.music_path), format)
    }
    fn load_sounds(&mut self, format: &cpal::Format, config: &Config) {
        println!("loading sounds");
        let mut cache = HashMap::new();
        // Take ownership of the Vec of hitsounds
        let self_sounds = std::mem::replace(&mut self.sounds, MaybeLoadedSounds::NotLoaded(vec![]));
        match self_sounds {
            MaybeLoadedSounds::NotLoaded(v) => {
                let mut loaded_sounds = Vec::with_capacity(v.len());
                for sounds in v {
                    let sound_results = sounds.into_iter().map(|s| s.load_sound(self, format, config, &mut cache));
                    let mut mixed_sound = audio::EffectStream::empty();
                    for sound_result in sound_results {
                        match sound_result {
                            Ok(sound) => mixed_sound = mixed_sound.mix(&sound),
                            Err((path, e)) => remani_warn!("Error loading hitsound '{}': {}", path.display(), e),
                        }
                    }
                    loaded_sounds.push(mixed_sound);
                }
                self.sounds = MaybeLoadedSounds::Loaded(loaded_sounds);
            }
            MaybeLoadedSounds::Loaded(..) => self.sounds = self_sounds,
        }
        let self_autoplay_sound_files = std::mem::replace(&mut self.autoplay_sound_files, MaybeLoadedAutoplaySounds::NotLoaded(vec![]));
        match self_autoplay_sound_files {
            MaybeLoadedAutoplaySounds::NotLoaded(v) => {
                let mut loaded_sounds = Vec::with_capacity(v.len());
                for path in v {
                    let mut path = self.chart_path.join(path);
                    let sound = if let Some(s) = cache.get(&path) {
                        s.clone()
                    } else {
                        let s = audio::music_from_path(&path, format)
                            .or_else(|_| {
                                path.set_extension("ogg");
                                audio::music_from_path(&path, format)
                            })
                            .or_else(|_| {
                                path.set_extension("mp3");
                                audio::music_from_path(&path, format)
                            })
                            .map(Into::into)
                            .unwrap_or_else(|e| {
                                remani_warn!("Error loading autoplay sound '{}': {}", path.display(), e);
                                audio::EffectStream::empty()
                            });

                        cache.insert(path, s.clone());
                        s
                    };
                    loaded_sounds.push(sound);
                }
                self.autoplay_sound_files = MaybeLoadedAutoplaySounds::Loaded(loaded_sounds);
            }
            MaybeLoadedAutoplaySounds::Loaded(..) => self.autoplay_sound_files = self_autoplay_sound_files,
        }
    }
    fn get_sound(&self, i: usize) -> Option<audio::EffectStream> {
        if i < self.sounds.len() {
            match &self.sounds {
                MaybeLoadedSounds::Loaded(v) => v.get(i).cloned(),
                MaybeLoadedSounds::NotLoaded(..) => {
                    remani_warn!("Hitsounds not loaded");
                    None
                }
            }
        } else {
            let i = i - self.sounds.len();
            match &self.autoplay_sound_files {
                MaybeLoadedAutoplaySounds::Loaded(v) => v.get(i).cloned(),
                MaybeLoadedAutoplaySounds::NotLoaded(..) => {
                    remani_warn!("Autoplay sounds not loaded");
                    None
                }
            }
        }
    }
}

impl IncompleteChart {
    fn finalize(self, chart_path: impl AsRef<Path>) -> Result<OsuChart, ParseError> {
        let mut sound_cache = HashMap::new();

        let timing_points = &self.timing_points;
        let notes: Vec<_> = self
            .hit_objects
            .into_iter()
            .map(|h| h.into_note(&mut sound_cache, timing_points))
            .collect();

        let mut sounds: Vec<_> = sound_cache.into_iter().collect();
        sounds.sort_unstable_by_key(|t| t.1);
        let sounds: Vec<_> = sounds.into_iter().map(|t| t.0).collect();

        let mut autoplay_sounds_cache = HashMap::new();
        let autoplay_sounds: Vec<_> = self.autoplay_sounds
            .into_iter()
            .map(|s| s.into_autoplay_sound(sounds.len(), &mut autoplay_sounds_cache))
            .collect();
        let mut autoplay_sound_files: Vec<_> = autoplay_sounds_cache.into_iter().collect();
        autoplay_sound_files.sort_unstable_by_key(|t| t.1);
        let autoplay_sound_files: Vec<_> = autoplay_sound_files.into_iter().map(|t| t.0).collect();

        let timing_points: Vec<_> = self
            .timing_points
            .into_iter()
            .map(|t| chart::TimingPoint {
                offset: t.offset,
                value: t.value,
            }).collect();


        let last_note_time = match notes.last() {
            Some(n) => n.end_time.unwrap_or(n.time),
            None => return Err(ParseError::Parse(String::from("Chart has no notes"), None)),
        };

        let primary_bpm = {
            // from beginning of song to the last note
            // sum of lengths of each bpm section
            let mut bpm_sums = Vec::new();
            let mut tp_iter = timing_points
                .iter()
                .filter(|tp| tp.is_bpm())
                .take_while(|tp| tp.offset < last_note_time)
                .peekable();

            if let Some(first_tp) = tp_iter.peek() {
                bpm_sums.push((first_tp.value.inner(), first_tp.offset));
            }

            while let Some(tp) = tp_iter.next() {
                let length = tp_iter.peek().map(|t| t.offset).unwrap_or(last_note_time) - tp.offset;

                if let Some(bpm_sum) = bpm_sums.iter_mut().find(|&&mut (bpm, _)| bpm == tp.value.inner()) {
                    bpm_sum.1 += length;
                } else {
                    bpm_sums.push((tp.value.inner(), length));
                }
            }

            // find the bpm that the song is at for the longest time, defaulting to 150 bpm if for
            // some reason that fails (FIXME?)
            bpm_sums
                .iter()
                .max_by(|(_, sum1), (_, sum2)| sum1.partial_cmp(sum2).unwrap_or(Ordering::Equal))
                .map(|t| t.0)
                .unwrap_or(150.0)
        };

        Ok(OsuChart {
            notes,
            timing_points,
            primary_bpm,
            autoplay_sounds,
            creator: self.creator,
            artist: self.artist,
            artist_unicode: self.artist_unicode,
            song_name: self.song_name,
            song_name_unicode: self.song_name_unicode,
            difficulty_name: self.difficulty_name.unwrap_or(String::from("Unnamed")),
            sounds: MaybeLoadedSounds::NotLoaded(sounds),
            autoplay_sound_files: MaybeLoadedAutoplaySounds::NotLoaded(autoplay_sound_files),

            music_path: self.music_path
                .ok_or(ParseError::Parse(String::from("Could not find audio file"), None))?,
            chart_path: chart_path.as_ref().to_owned(),
        })
    }
}

/// Represents a timing point from the Timing Points section of the .osu chart. This has extra
/// stuff that we need but does not go into the real TimingPoint
#[derive(Debug)]
struct OsuTimingPoint {
    offset: f64,
    value: chart::TimingPointValue,
    sample_set: SampleSet,
    sample_index: u32,
    volume: u8,
}

/// Parses .osu charts and returns a `Chart`
#[derive(Default)]
struct OsuParser {
    current_section: Option<String>,
    chart: IncompleteChart,
}

impl OsuParser {
    fn parse_line(&mut self, line: &str) -> Result<(), ParseError> {
        if line.is_empty() {
            return Ok(());
        }
        match &line[0..1] {
            "[" => self.current_section = Some(parse_section(line).to_owned()),

            _ => match self.current_section {
                Some(ref s) => match s.as_str() {
                    "General" => parse_general(line, &mut self.chart)?,
                    "Difficulty" => parse_difficulty(line)?,
                    "Metadata" => parse_metadata(line, &mut self.chart)?,
                    "TimingPoints" => parse_timing_point(line, &mut self.chart)?,
                    "HitObjects" => parse_hit_object(line, &mut self.chart)?,
                    "Events" => parse_events(line, &mut self.chart)?,
                    _ => (),
                },
                None => return Err(ParseError::InvalidFile),
            },
        }
        Ok(())
    }
}

/// Takes a path to the .osu file
pub fn from_path<P: AsRef<Path>>(path: P) -> Result<impl Chart, ParseError> {
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            return Err(ParseError::Io(format!("Error opening {}", path.as_ref().display()), e))
        }
    };
    let reader = io::BufReader::new(file);
    let mut parser = OsuParser::default();
    macro_rules! read_error {
        ($e:expr) => {
            Err(ParseError::Io(String::from("Error reading chart"), $e))
        };
    }

    let mut lines = reader.lines();
    let line = match lines.next() {
        Some(r) => match r {
            Ok(s) => s,
            Err(e) => return read_error!(e),
        },
        None => return Err(ParseError::InvalidFile),
    };

    let version = verify(line.as_str())?;
    println!("File Format Version {}", version);

    for (line_num, line) in lines.enumerate() {
        match line {
            Ok(line) => cvt_err!(
                // + 1 because 0 based index, + 1 for the line we read earlier
                format!("Error on line {} of .osu file", line_num + 2),
                parser.parse_line(line.trim())
            )?,
            Err(e) => return read_error!(e),
        }
    }
    // this unwrap shouldn't ever panic since an error would've been returned from trying to open
    // the file earlier
    println!("{}", path.as_ref().parent().unwrap().display());
    Ok(parser.chart.finalize(path.as_ref().parent().unwrap())?)
}

#[cfg(test)]
mod tests {
    use crate::chart::osu::*;

    /// Test hit object parser
    #[test]
    fn test_ho_parse() {
        let mut chart = IncompleteChart::default();
        parse_hit_object("0,0,5000,128,0,6000:0:0:0:70:", &mut chart).expect("Failed to parse hit object");
        {
            let ho = &chart.hit_objects[0];
            assert_eq!(5.0, ho.time);
            assert_eq!(0, ho.column);
            assert_eq!(70, ho.sounds[0].volume);
            assert_eq!(Some(6.0), ho.end_time);
            match ho.sounds[0].source {
                HitSoundSource::SampleSet(ref shs) => {
                    assert_eq!(SampleSet::Auto, shs.set);
                    assert_eq!(SampleHitSoundSound::Normal, shs.sound);
                    assert_eq!(0, shs.index);
                }
                _ => panic!("Incorrect hit sound source"),
            }
        }

        chart.hit_objects.clear();

        parse_hit_object("75,0,1337,0,0,0:0:0:10:potato.wav", &mut chart).expect("Failed to parse hit object");
        {
            let ho = &chart.hit_objects[0];
            assert_eq!(1.337, ho.time);
            assert_eq!(1, ho.column);
            assert_eq!(10, ho.sounds[0].volume);
            match ho.sounds[1].source {
                HitSoundSource::File(ref path) => {
                    assert_eq!("potato.wav", path.to_str().unwrap());
                }
                _ => panic!("Incorrect hit sound source"),
            }
        }
    }
}
