//! O2Jam chart parser module

use nom::*;

use std::io::Read;

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

fn string_from_slice(s: &[u8]) -> String {
    String::from_utf8_lossy(s).into_owned()
}

macro_rules! string_block {
    ($i:expr, $n:expr) => (flat_map!($i, take!($n), map!(take_until!("\0"), string_from_slice)));
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

use std::path::Path;
use std::fs::File;

pub fn dump_header<P: AsRef<Path>>(path: P) {
    let mut buffer = [0; 300];
    let mut file = File::open(path).expect("Failed to open ojn file");
    file.read_exact(&mut buffer).expect("error reading ojn file");
    println!("{:#?}", header(&buffer));
}
