//! O2Jam chart parser module

use nom::*;

use crate::chart::{Chart, Note, TimingPoint, TimingPointValue};

use std::fmt;

fn string_from_slice(s: &[u8]) -> String {
    String::from_utf8_lossy(s).into_owned()
}

macro_rules! string_block {
    ($i:expr, $n:expr) => (flat_map!($i, take!($n), map!(take_until!("\0"), string_from_slice)));
}

mod ojm;

// TODO temporary for testing
pub use self::ojm::dump_data as ojm_dump;

#[derive(Debug)]
struct Header {
    songid: i32,
    //signature: [u8; 4],
    encode_version: f32,
    genre: i32,
    bpm: f32,
    level: [i16; 4],
    event_count: [i32; 3],
    note_count: [i32; 3],
    measure_count: [i32; 3],
    package_count: [i32; 3],
    old_encode_version: i16,
    old_songid: i16,
    old_genre: String,
    bmp_size: i32,
    old_file_version: i32,
    title: String,
    artist: String,
    noter: String,
    ojm_file: String,
    cover_size: i32,
    time: [i32; 3],
    note_offset: [i32; 3],
    cover_offset: i32,
}

named!(header(&[u8]) -> Header,
    do_parse!(
        songid: le_i32  >>
        tag!("ojn\0") >>
        encode_version: le_f32 >>
        genre: le_i32 >>
        bpm: le_f32 >>
        level: count_fixed!(i16, le_i16, 4) >>
        event_count: count_fixed!(i32, le_i32, 3) >>
        note_count: count_fixed!(i32, le_i32, 3) >>
        measure_count: count_fixed!(i32, le_i32, 3) >>
        package_count: count_fixed!(i32, le_i32, 3) >>
        old_encode_version: le_i16 >>
        old_songid: le_i16 >>
        old_genre: string_block!(20) >>
        bmp_size: le_i32 >>
        old_file_version: le_i32 >>
        title: string_block!(64) >>
        artist: string_block!(32) >>
        noter: string_block!(32) >>
        ojm_file: string_block!(32) >>
        cover_size: le_i32 >>
        time: count_fixed!(i32, le_i32, 3) >>
        note_offset: count_fixed!(i32, le_i32, 3) >>
        cover_offset: le_i32 >>
        (Header {
            songid,
            encode_version,
            genre,
            bpm,
            level,
            event_count,
            note_count,
            measure_count,
            package_count,
            old_encode_version,
            old_songid,
            old_genre,
            bmp_size,
            old_file_version,
            title,
            artist,
            noter,
            ojm_file,
            cover_size,
            time,
            note_offset,
            cover_offset,
        })
    )
);

/// Documentation largely copied from https://open2jam.wordpress.com/2010/10/05/the-notes-section/
#[derive(Debug)]
struct PackageHeader {
    /// This is the measure in which the events inside this package will appear.
    measure: i32,
    /// channel meaning
    ///
    /// 0 measure fraction
    ///
    /// 1 BPM change
    ///
    /// 2 note on 1st lane
    ///
    /// 3 note on 2nd lane
    ///
    /// 4 note on 3rd lane
    ///
    /// 5 note on 4th lane(middle button)
    ///
    /// 6 note on 5th lane
    ///
    /// 7 note on 6th lane
    ///
    /// 8 note on 7th lane
    ///
    /// 9~22 auto-play samples(?)
    channel: i16,
    /// The number of events inside this package
    events: i16,
}

named!(package_header(&[u8]) -> PackageHeader,
    do_parse!(
        measure: le_i32 >>
        channel: le_i16 >>
        events: le_i16 >>
        (PackageHeader { measure, channel, events })
    )
);

/// Documentation largely copied from https://open2jam.wordpress.com/2010/10/05/the-notes-section/
#[derive(Debug)]
struct NoteEvent {
    /// Reference to the sample in the OJM file, unless this value is 0, in which case the entire
    /// event is ignored
    value: i16,
    /// 0..=15, 0 is max volume,
    volume: u8,
    /// The panning of the sample, although this can also be controlled using stereo samples,
    /// further control can be given by using different pans with the same sample (I guess).
    ///
    /// 1~7 = left -> center, 0 or 8 = center, 9~15 = center -> right.
    pan: u8,
    /// 0 = normal note
    ///
    /// 2 = long note start
    ///
    /// 3 = long note end
    ///
    /// 4 = "OGG sample" (???)
    note_type: u8,
}

#[derive(Debug)]
enum Events {
    MeasureFraction(Vec<f32>),
    BpmChange(Vec<f32>),
    /// The first `usize` specifies the column
    NoteEvent(usize, Vec<NoteEvent>),
    /// The first i16 specifies the event id
    Unknown(i16, Vec<[u8; 4]>),
}

named!(note_event(&[u8]) -> NoteEvent,
    do_parse!(
        value: le_i16 >>
        volume_pan: le_u8 >>
        note_type: le_u8 >>
        (NoteEvent { value, volume: volume_pan >> 4, pan: volume_pan & 0xF, note_type })
    )
);

#[derive(Debug)]
struct Package {
    measure: i32,
    events: Events,
}

fn events(input: &[u8], channel: i16, event_count: i16) -> IResult<&[u8], Events> {
    let event_count = event_count as usize;
    match channel {
        0 => map!(input, count!(le_f32, event_count), |v| Events::MeasureFraction(v)),
        1 => map!(input, count!(le_f32, event_count), |v| Events::BpmChange(v)),
        n @ 2..=8 => map!(input, count!(note_event, event_count), |v| Events::NoteEvent(n as usize, v)),
        n => map!(input, count!(count_fixed!(u8, le_u8, 4), event_count), |v| Events::Unknown(n, v)),
    }
}

named!(package(&[u8]) -> Package,
    do_parse!(
        header: package_header >>
        events: apply!(events, header.channel, header.events) >>
        (Package { measure: header.measure, events })
    )
);

fn notes_section(input: &[u8], package_count: usize) -> IResult<&[u8], Vec<Package>> {
    count!(input, package, package_count)
}

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl From<Difficulty> for &'static str {
    fn from(t: Difficulty) -> &'static str {
        match t {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
        }
    }
}

// TODO
struct O2mChart {
    notes: Vec<Note>,
    bpm_changes: Vec<TimingPoint>,
    creator: String,
    artist: String,
    song_name: String,
    difficulty: Difficulty,
}

fn print_note_count(packages: &[Package]) {
    let note_count = packages.iter()
        .filter_map(|p| match &p.events {
            Events::NoteEvent(_, v) => Some(v),
            _ => None,
        })
        .flatten()
        .count();

    println!("Note count: {}", note_count);
}

pub fn dump_data<P: AsRef<Path>>(path: P) {
    let mut hdr_buffer = [0; 300];
    let mut file = File::open(path).expect("Failed to open ojn file");
    file.read_exact(&mut hdr_buffer).expect("error reading ojn file");
    let (_, hdr) = header(&hdr_buffer).unwrap();
    // println!("header: {:#?}", hdr);

    let easy_len = (hdr.note_offset[1] - hdr.note_offset[0]) as usize;
    let normal_len = (hdr.note_offset[2] - hdr.note_offset[1]) as usize;
    let hard_len = (hdr.cover_offset - hdr.note_offset[2]) as usize;
    let mut notesection_buffer = vec![0; easy_len.max(normal_len).max(hard_len)];

    file.seek(SeekFrom::Start(hdr.note_offset[0] as u64)).unwrap();
    file.read_exact(&mut notesection_buffer[0..easy_len]).expect("Error reading ojn file");
    let (_, easy_packages) = notes_section(&notesection_buffer, hdr.package_count[0] as usize).unwrap();

    file.seek(SeekFrom::Start(hdr.note_offset[1] as u64)).unwrap();
    file.read_exact(&mut notesection_buffer[0..normal_len]).expect("Error reading ojn file");
    let (_, normal_packages) = notes_section(&notesection_buffer, hdr.package_count[1] as usize).unwrap();

    file.seek(SeekFrom::Start(hdr.note_offset[2] as u64)).unwrap();
    file.read_exact(&mut notesection_buffer[0..hard_len]).expect("Error reading ojn file");
    let (_, hard_packages) = notes_section(&notesection_buffer, hdr.package_count[2] as usize).unwrap();

    println!("Easy difficulty info");
    println!("Length: {}:{:02}", hdr.time[0] / 60, hdr.time[0] % 60);
    println!("Level: {}", hdr.level[0]);
    print_note_count(&easy_packages);

    println!();

    println!("Normal difficulty info");
    println!("Length: {}:{:02}", hdr.time[1] / 60, hdr.time[1] % 60);
    println!("Level: {}", hdr.level[1]);
    print_note_count(&normal_packages);

    println!();

    println!("Hard difficulty info");
    println!("Length: {}:{:02}", hdr.time[2] / 60, hdr.time[2] % 60);
    println!("Level: {}", hdr.level[2]);
    print_note_count(&hard_packages);
}
