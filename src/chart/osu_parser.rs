//! Osu chart parser module

use std::io;
use std::path::PathBuf;
use std::cmp::Ordering;

use chart;
use chart::{ Chart, ChartParser, ParseError };
use chart::Note;

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

/// Parse a line from the General section and add the info to the chart passed in
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

/// Parse a line from the Metadata section and add the info to the chart passed in
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

/// Parse a line from the Difficulty section. This just checks to see if the chart is a 7k chart.
fn parse_difficulty(line: &str) -> Result<(), ParseError> {
    let (k, v) = line.split_at(match line.find(':') {
        Some(n) => n,
        None => return Err(ParseError::Parse(String::from("Malformed key/value pair"), None)),
    });
    let v = &v[1..];

    if k == "CircleSize" && v != "7" {
            Err(ParseError::Parse(String::from("This chart is not 7 key"), None))
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
            },

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
                    x => { println!("Unknown sample set {}", x); SampleSet::Auto },
                });
            },

            // sample index
            4 => {
                sample_index = Some(cvt_err!(ERR_STRING, field.parse::<u32>())?);
            },

            // volume
            5 => {
                volume = Some(cvt_err!(ERR_STRING, field.parse::<u8>())?);
            },

            // inherited, we're gonna ignore since we determined this when looking at the ms / beat
            6 => (),
            // kiai mode, not important
            7 => (),
            _ => unreachable!(),
        }
    }
    if last_index < 7 {
        return Err(ParseError::Parse(ERR_STRING.to_owned(),
                                     Some(Box::new(ParseError::EOL))));
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
    const ERR_STRING: &'static str = "Error parsing hit object";

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
                if c < 0.0 { c = 0.0; }
                else if c > 7.0 { c = 7.0; }
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
                    }
                }
                hit_obj.sounds.push(dflt_hit_snd!(SampleHitSoundSound::Normal));
                if n & 2 == 2 { hit_obj.sounds.push(dflt_hit_snd!(SampleHitSoundSound::Whistle)); }
                if n & 4 == 4 { hit_obj.sounds.push(dflt_hit_snd!(SampleHitSoundSound::Finish)); }
                if n & 8 == 8 { hit_obj.sounds.push(dflt_hit_snd!(SampleHitSoundSound::Clap)); }
            },
            // endtime/extras
            5 => {
                let mut extras = field.split(':');
                if ln {
                    hit_obj.end_time = Some(match extras.next() {
                        Some(s) => cvt_err!(ERR_STRING, s.parse::<f64>())? / 1000.0,
                        None => return Err(ParseError::Parse(ERR_STRING.to_owned(), Some(Box::new(ParseError::EOL)))),
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
                                1 => hs_iter.for_each(|s| if let HitSoundSource::SampleSet(ref mut shs) = s.source { shs.set = SampleSet::Normal }),
                                2 => hs_iter.for_each(|s| if let HitSoundSource::SampleSet(ref mut shs) = s.source { shs.set = SampleSet::Soft }),
                                3 => hs_iter.for_each(|s| if let HitSoundSource::SampleSet(ref mut shs) = s.source { shs.set = SampleSet::Drum }),
                                _ => (),
                            }
                        },
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
                                1 => hs_iter.for_each(|s| if let HitSoundSource::SampleSet(ref mut shs) = s.source { shs.set = SampleSet::Normal }),
                                2 => hs_iter.for_each(|s| if let HitSoundSource::SampleSet(ref mut shs) = s.source { shs.set = SampleSet::Soft }),
                                3 => hs_iter.for_each(|s| if let HitSoundSource::SampleSet(ref mut shs) = s.source { shs.set = SampleSet::Drum }),
                                _ => (),
                            }
                        },
                        // custom index
                        2 => {
                            match cvt_err!(ERR_STRING, v.parse::<u32>())? {
                                0 => (),
                                n => {
                                    hit_obj.sounds.iter_mut().for_each(|s| if let HitSoundSource::SampleSet(ref mut shs) = s.source { shs.index = n });
                                }
                            }
                        },
                        // volume
                        3 => {
                            let n = cvt_err!(ERR_STRING, v.parse::<u8>())?;
                            if n != 0 {
                                hit_obj.sounds.iter_mut().for_each(|s| s.volume = n);
                                volume = n;
                            }
                        },
                        // hitsound from file
                        4 => if !v.is_empty() {
                            hit_obj.sounds.push(HitSound {
                                volume: volume,
                                source: HitSoundSource::File(PathBuf::from(v)),
                            });
                        },
                        _ => unreachable!(),
                    }
                }
            }
            _ => unreachable!()
        }
    }
    if last_index < 5 { return Err(ParseError::Parse(ERR_STRING.to_owned(),
                            Some(Box::new(ParseError::EOL)))); }

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

    //fn to_note(self, sound: Rc<something>) {
    fn to_note(self) -> Note {
        Note {
            time: self.time,
            column: self.column,
            end_time: self.end_time,
            //sound: mixed?_hitsounds,
            // TODO
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
    index: u32,
}

/// A sample set
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum SampleSet {
    Auto,
    Normal,
    Soft,
    Drum,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
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
    timing_points: Vec<OsuTimingPoint>,
    creator: Option<String>,
    artist: Option<String>,
    artist_unicode: Option<String>,
    song_name: Option<String>,
    song_name_unicode: Option<String>,
    difficulty_name: Option<String>,
    music_path: Option<PathBuf>,
}

impl IncompleteChart {
    fn finalize(self) -> Result<Chart, ParseError> {
        let timing_points = self.timing_points.into_iter()
                            .map(|t| chart::TimingPoint {
                                offset: t.offset,
                                value: t.value,
                            }).collect::<Vec<_>>();

        let notes = self.hit_objects.into_iter().map(HitObject::to_note).collect::<Vec<_>>();

        let last_note_time = match notes.last() {
            Some(n) => n.end_time.unwrap_or(n.time),
            None => return Err(
                ParseError::Parse(String::from("Chart has no notes"), None)),
        };

        let primary_bpm = {

            // from beginning of song to the last note
            // sum of lengths of each bpm section
            let mut bpm_sums = Vec::new();
            let mut tp_iter = timing_points.iter().filter(|tp| tp.is_bpm()).take_while(|tp| tp.offset < last_note_time).peekable();

            if let Some(first_tp) = tp_iter.peek() {
                bpm_sums.push((first_tp.value.unwrap(), first_tp.offset));
            }

            while let Some(tp) = tp_iter.next() {
                let length = tp_iter.peek().map(|t| t.offset).unwrap_or(last_note_time) - tp.offset;

                // rust pls fix borrow checker
                // if let Some(bpm_sum) = bpm_sums.iter_mut().find(|&&mut (bpm, _)| bpm == tp.value.unwrap()) {
                //     bpm_sum.1 += length;
                // } else {
                //     bpm_sums.push((tp.value.unwrap(), length));
                // }

                if !{ if let Some(bpm_sum) = bpm_sums.iter_mut().find(|&&mut (bpm, _)| bpm == tp.value.unwrap()) {
                        bpm_sum.1 += length;
                        true
                    } else {
                        false
                    }} { // im dying
                    bpm_sums.push((tp.value.unwrap(), length));
                }

            }

            // find the bpm that the song is at for the longest time, defaulting to 150 bpm if for
            // some reason that fails (FIXME?)
            bpm_sums.iter().max_by(|(_, sum1), (_, sum2)| sum1.partial_cmp(sum2).unwrap_or(Ordering::Equal))
                .map(|t| t.0).unwrap_or(150.0)
        };

        Ok(Chart {
            notes,
            timing_points,
            primary_bpm,
            creator: self.creator,
            artist: self.artist,
            artist_unicode: self.artist_unicode,
            song_name: self.song_name,
            song_name_unicode: self.song_name_unicode,
            difficulty_name: self.difficulty_name.unwrap_or(String::from("Unnamed")),

            music_path: match self.music_path {
                Some(s) => s,
                None => return Err(
                    ParseError::Parse(String::from("Could not find audio file"), None)),
            }
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
pub struct OsuParser {
    current_section: Option<String>,
    chart: IncompleteChart,
}

impl OsuParser {

    fn parse_line(&mut self, line: &str) -> Result<(), ParseError> {
        if line.len() == 0 { return Ok(()); }
        match &line[0..1] {

            "[" => self.current_section = Some(parse_section(line).to_owned()),

            _ => match self.current_section {

                Some(ref s) => match s.as_str() {
                    "General" => parse_general(line, &mut self.chart)?,
                    "Difficulty" => parse_difficulty(line)?,
                    "Metadata" => parse_metadata(line, &mut self.chart)?,
                    "TimingPoints" => parse_timing_point(line, &mut self.chart)?,
                    "HitObjects" => parse_hit_object(line, &mut self.chart)?,
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

        macro_rules! read_error {
            ($e:expr) =>  {
                Err(ParseError::Io(String::from("Error reading chart"), $e))
            }
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
                Ok(line) => cvt_err!(format!("Error on line {} of .osu file", line_num + 2), self.parse_line(line.trim()))?,
                Err(e) => return read_error!(e),
            }
        }

        Ok(self.chart.finalize()?)
    }
}

#[cfg(test)]
mod tests {
    use chart::osu_parser::*;

    /// Test hit object parser
    #[test]
    fn test_ho_parse() {
        let mut chart = IncompleteChart::default();
        parse_hit_object("0,0,5000,128,0,6000:0:0:0:70:", &mut chart);
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
                },
                _ => panic!("Incorrect hit sound source"),
            }
        }

        chart.hit_objects.clear();

        parse_hit_object("75,0,1337,0,0,0:0:0:10:potato.wav", &mut chart);
        {
            let ho = &chart.hit_objects[0];
            assert_eq!(1.337, ho.time);
            assert_eq!(1, ho.column);
            assert_eq!(10, ho.sounds[0].volume);
            match ho.sounds[1].source {
                HitSoundSource::File(ref path) => {
                    assert_eq!("potato.wav", path.to_str().unwrap());
                },
                _ => panic!("Incorrect hit sound source"),
            }
        }
    }
}
