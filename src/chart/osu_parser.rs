//! Osu chart parser module

use std::io;
use std::io::BufRead;
use std::path::PathBuf;

use chart::{ Chart, ChartParser, ParseError };
use chart::{ Note, TimingPoint, BPM, SV };

/// Convert Err values to ParseError
macro_rules! cvt_err {
    ($s:expr, $e:expr) => {
        $e.or_else(|e| Err(ParseError::Parse($s.to_owned(), Some(Box::new(e)))))
    }
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
    &line[1..line.len()-1]
}

/// Parse a line from the General section
fn parse_general(line: &str, chart: &mut IncompleteChart) -> Result<(), ParseError> {

    let (k, v) = line.split_at(match line.find(':') {
        Some(n) => n,
        None => return Err(ParseError::Parse(String::from("Error parsing General section: Malformed key/value pair"), None)),
    });
    let v = &v[2..];

    match k {
        "AudioFilename" => chart.music_path = Some(v.into()),
        "Mode" => if v != "3" {
            return Err(ParseError::Parse(
                    String::from("Osu chart is wrong gamemode"), None));
        },
        _ => (),
    }
    Ok(())
}

/// Parse a line from the Metadata section
fn parse_metadata(line: &str, chart: &mut IncompleteChart) -> Result<(), ParseError> {
    let (k, v) = line.split_at(match line.find(':') {
        Some(n) => n,
        None => return Err(ParseError::Parse(String::from("Malformed key/value pair"), None)),
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

/// Parse a line from the TimingPoints section and returns the index of the last bpm change (not
/// sv)
fn parse_timing_points(line: &str, chart: &mut IncompleteChart, last_bpm_change_index: Option<usize>) -> Result<usize, ParseError> {

    static ERR_STRING: &str = "Error parsing timing points";

    let mut last_index = 0;

    let mut offset: Option<f64> = None;
    let mut bpm: Option<f64> = None;
    let mut sv: Option<f64> = None;

    let mut absolute = true;

    for (index, field) in line.split(',').enumerate().take(8) {

        // Keep track of how many fields there were
        last_index = index;

        let n = cvt_err!(ERR_STRING, field.parse::<f64>())?;

        match index {
            0 => offset = Some(n / 1000.0),
            1 => if n.is_sign_positive() {

                bpm = Some(60000.0 / n);

            } else {

                sv = Some(100.0 / -n);
                absolute = false;

            },
            _ => (),
        }
    }
    if last_index < 7 {
        return Err(ParseError::Parse(ERR_STRING.to_owned(),
                                     Some(Box::new(ParseError::EOL))));
    }

    if absolute {

        let timing_point = TimingPoint::BPM(BPM { offset: offset.unwrap(), bpm: bpm.unwrap() });
        chart.timing_points.push(timing_point);

        Ok(chart.timing_points.len() - 1)
    } else {

        let timing_point = TimingPoint::SV(SV { offset: offset.unwrap(), sv: sv.unwrap() });
        chart.timing_points.push(timing_point);

        Ok(last_bpm_change_index.unwrap())
    }
}


/// Parse a line from the HitObjects section
fn parse_hit_object(line: &str, chart: &mut IncompleteChart) -> Result<(), ParseError> {

    let mut last_index = 0;
    const ERR_STRING: &'static str = "Error parsing hit object";

    let mut hit_obj = HitObject::default();
    for (index, field) in line.split(',').enumerate().take(6) {

        // Keep track of how many fields there were
        last_index = index;

        let mut ln = false;

        match index {
            0 => {
                // calculate column

                let n = cvt_err!(ERR_STRING, field.parse::<f64>())?;
                const cw: f64 = 512.0 / 7.0;
                let mut c = (n / cw).floor();
                if c < 0.0 { c = 0.0; }
                else if c > 7.0 { c = 7.0; }
                hit_obj.column = c as u8;
            }
            1 => (),
            2 => hit_obj.time = cvt_err!(ERR_STRING, field.parse::<f64>())? / 1000.0,
            3 => ln = cvt_err!(ERR_STRING, field.parse::<u8>())? & 0xE == 0xE,
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
                                custom_index: 0,
                            }),
                        }
                    }
                }
                if n & 2 == 2 { hit_obj.sounds.push(dflt_hit_snd!(SampleHitSoundSound::Whistle)); }
                if n & 4 == 4 { hit_obj.sounds.push(dflt_hit_snd!(SampleHitSoundSound::Finish)); }
                if n & 8 == 8 { hit_obj.sounds.push(dflt_hit_snd!(SampleHitSoundSound::Clap)); }
            },
            5 => {
                let mut extras = field.split(':');
                if ln {
                    hit_obj.end_time = Some(match extras.next() {
                        Some(s) => cvt_err!(ERR_STRING, s.parse::<f64>())? / 1000.0,
                        None => return Err(ParseError::Parse(ERR_STRING.to_owned(), Some(Box::new(ParseError::EOL)))),
                    });
                }
                for (i, v) in extras.enumerate().take(5) {
                    let mut volume = 100;
                    match i {
                        0 => {
                            match cvt_err!(ERR_STRING, v.parse::<u8>())? {
                                0 => (),
                                1 => hit_obj.sounds.iter_mut().for_each(|s| if let HitSoundSource::SampleSet(ref mut shs) = s.source { shs.set = SampleSet::Normal }),
                                2 => hit_obj.sounds.iter_mut().for_each(|s| if let HitSoundSource::SampleSet(ref mut shs) = s.source { shs.set = SampleSet::Soft }),
                                3 => hit_obj.sounds.iter_mut().for_each(|s| if let HitSoundSource::SampleSet(ref mut shs) = s.source { shs.set = SampleSet::Drum }),
                                _ => (),
                            }
                        },
                        1 => (),
                        2 => {
                            match cvt_err!(ERR_STRING, v.parse::<u32>())? {
                                0 => (),
                                n => {
                                    hit_obj.sounds.iter_mut().for_each(|s| if let HitSoundSource::SampleSet(ref mut shs) = s.source { shs.custom_index = n });
                                }
                            }
                        },
                        3 => {
                            let n = cvt_err!(ERR_STRING, v.parse::<u8>())?;
                            if n != 0 {
                                hit_obj.sounds.iter_mut().for_each(|s| s.volume = n);
                                volume = n;
                            }
                        },
                        4 => {
                            hit_obj.sounds = vec![ HitSound {
                                volume: volume,
                                source: HitSoundSource::File(PathBuf::from(v)),
                            } ];
                        },
                        _ => unreachable!(),
                    }
                }
            }
            _ => unreachable!()
        }
    }
    chart.hit_objects.push(hit_obj);
    Ok(())
}

/// Represents a hit object. This will get converted into a Note once the file is parsed and we can
/// get the audio samples for the hit sound.
#[derive(Default,Debug)]
struct HitObject {

    /// Where the note begins, in seconds.
    time: f64,
    column: u8,
    end_time: Option<f64>,
    sounds: Vec<HitSound>,
}

impl HitObject {

    //fn to_note(self, sound: Rc<something>) {
    fn to_note(self) -> Note {
        Note {
            time: self.time,
            column: self.column,
            end_time: self.end_time,
            //sound: mixed?_hitsounds,
        }
    }
}

#[derive(Debug)]
struct HitSound {

    /// The audio source of the sample.
    source: HitSoundSource,

    /// The volume of the hit sound, from 0 to 100
    volume: u8,
}

/// Where to get the audio source of the hit sound
#[derive(Debug)]
enum HitSoundSource {
    SampleSet(SampleHitSound),
    File(PathBuf),
}

/// A hit sound that comes from a sample set
#[derive(Debug)]
struct SampleHitSound {
    set: SampleSet,
    sound: SampleHitSoundSound,
    custom_index: u32,
}

/// A sample set
#[derive(Debug)]
enum SampleSet {
    Auto,
    Normal,
    Soft,
    Drum,
}

#[derive(Debug)]
enum SampleHitSoundSound {
    Normal,
    Whistle,
    Finish,
    Clap,
}

/// Used during parsing, gets finalized into a Chart once all the values are obtained.
#[derive(Default)]
struct IncompleteChart {
    hit_objects: Vec<HitObject>,
    timing_points: Vec<TimingPoint>,
    creator: Option<String>,
    artist: Option<String>,
    artist_unicode: Option<String>,
    song_name: Option<String>,
    song_name_unicode: Option<String>,
    difficulty_name: Option<String>,
    music_path: Option<PathBuf>,
}

/// Parses .osu charts and returns a `Chart`
#[derive(Default)]
pub struct OsuParser {
    current_section: Option<String>,
    chart: IncompleteChart,
    last_bpm_change_index: Option<usize>,
}

impl OsuParser {

    fn parse_line(&mut self, line: &str) -> Result<(), ParseError> {
        if line.len() == 0 { return Ok(()); }
        match &line[0..1] {

            "[" => self.current_section = Some(parse_section(line).to_owned()),

            _ => match self.current_section {

                Some(ref s) => match s.as_str() {
                    "General" => parse_general(line, &mut self.chart)?,
                    "Metadata" => parse_metadata(line, &mut self.chart)?,
                    "TimingPoints" => self.last_bpm_change_index = Some(parse_timing_points(line, &mut self.chart, self.last_bpm_change_index)?),
                    _ => (),
                },
                None => return Err(ParseError::InvalidFile),
            },
        }
        Ok(())
    }
}

impl ChartParser for OsuParser {
    fn parse<R: io::BufRead>(mut self, reader: R) -> Result<Chart, ParseError> {

        let read_error = |e| Err(ParseError::Io(String::from("Error reading chart"), e));

        let mut lines = reader.lines();
        let line = match lines.next() {
            Some(r) => match r {
                Ok(s) => s,
                Err(e) => return read_error(e),
            },
            None => return Err(ParseError::InvalidFile),
        };

        for line in lines {
            match line {
                Ok(line) => self.parse_line(line.trim())?,
                Err(e) => return read_error(e),
            }
        }

        Ok(Chart {

            music_path: match self.chart.music_path {
                Some(s) => s,
                None => return Err(
                    ParseError::Parse(String::from("Could not find audio file"), None)),
            },

            //notes: self.chart.notes,
            notes: Default::default(),
            timing_points: self.chart.timing_points,
            creator: self.chart.creator,
            artist: self.chart.artist,
            artist_unicode: self.chart.artist_unicode,
            song_name: self.chart.song_name,
            song_name_unicode: self.chart.song_name_unicode,
            difficulty_name: self.chart.difficulty_name.unwrap_or(String::from("Unnamed")),
        })
    }
}
