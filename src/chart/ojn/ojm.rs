//! O2Jam sound file parser module

use nom::*;

use super::string_from_slice;
use crate::audio;

use std::iter;

/// Header of the OMC/OJM format
#[derive(Debug)]
struct OmcHeader {
    encrypted: bool,
    wav_count: i16,
    ogg_count: i16,
    wav_start: i32,
    ogg_start: i32,
}

named!(omc_header(&[u8]) -> OmcHeader,
    do_parse!(
        encrypted: alt!(
            tag!("OJM\0") => { |_| false } |
            tag!("OMC\0") => { |_| true }
        ) >>
        wav_count: le_i16 >>
        ogg_count: le_i16 >>
        wav_start: le_i32 >>
        ogg_start: le_i32 >>
        take!(4) >> // filesize that we don't need
        (OmcHeader { encrypted, wav_count, ogg_count, wav_start, ogg_start })
    )
);

#[derive(Debug)]
enum M30Encryption {
    Scramble1,
    Scramble2,
    Decode,
    Decrypt,
    XorNami,
    Xor0412,
    Unknown,
}

impl From<i32> for M30Encryption {
    fn from(i: i32) -> Self {
        match i {
            1 => M30Encryption::Scramble1,
            2 => M30Encryption::Scramble2,
            4 => M30Encryption::Decode,
            8 => M30Encryption::Decrypt,
            16 => M30Encryption::XorNami,
            32 => M30Encryption::Xor0412,
            _ => M30Encryption::Unknown,
        }
    }
}

fn m30_encryption_from_i32(i: i32) -> M30Encryption {
    i.into()
}

named!(m30_encryption(&[u8]) -> M30Encryption,
    map!(le_i32, m30_encryption_from_i32)
);

/// Header of the M30 format
#[derive(Debug)]
struct M30Header {
    file_format_version: i32,
    encryption_scheme: M30Encryption,
    samples_offset: i32,
}

named!(m30_header(&[u8]) -> M30Header,
    do_parse!(
        tag!("M30\0") >>
        file_format_version: le_i32 >>
        encryption_scheme: m30_encryption >>
        take!(4) >> // sample_count which is apparently unreliable
        samples_offset: le_i32 >>
        take!(4) >> // payload size
        take!(4) >> // padding
       (M30Header { file_format_version, encryption_scheme, samples_offset })
    )
);

#[derive(Debug)]
enum Header {
    M30(M30Header),
    Omc(OmcHeader),
}

named!(header(&[u8]) -> Header,
    alt!(
        omc_header => { |h| Header::Omc(h) } |
        m30_header => { |h| Header::M30(h) }
    )
);

fn m30_decrypt(data: &[u8], hdr: &M30Header) -> Vec<u8> {
    match hdr.encryption_scheme {

        M30Encryption::XorNami => data
            .chunks_exact(4)
            .flatten()
            .zip(iter::repeat(b"nami").flatten())
            .map(|(a, b)| a^b)
            .collect(),

        M30Encryption::Xor0412 => data
            .chunks_exact(4)
            .flatten()
            .zip(iter::repeat(b"0412").flatten())
            .map(|(a, b)| a^b)
            .collect(),
        _ => {
            // We don't actually know how the others work, and apparently files using the other
            // ones haven't been seen so ¯\_(ツ)_/¯
            remani_warn!("Unrecognized M30 encryption scheme");
            Vec::from(data)
        }
    }
}

#[derive(Debug)]
struct M30Sound {
    sound_name: String,
    sound_size: i32,
    codec_code: i16,
    note_ref: i16,
    sample_count: i32,
    ogg_data: Vec<u8>,
}

fn m30_sound<'a>(input: &'a [u8], hdr: &M30Header) -> IResult<&'a [u8], M30Sound> {
    do_parse!(input,
        sound_name: string_block!(32) >>
        sound_size: le_i32 >>
        codec_code: le_i16 >>
        take!(2) >> // unk_fixed
        take!(4) >> // unk_music_flag
        note_ref: le_i16 >>
        take!(2) >> // unk_zero
        sample_count: le_i32 >>
        ogg_data: take!(sound_size) >>
        (M30Sound {
            sound_name,
            sound_size,
            codec_code,
            note_ref,
            sample_count,
            ogg_data: m30_decrypt(ogg_data, hdr),
        })
    )
}

fn m30_sounds<'a>(input: &'a [u8], hdr: &M30Header) -> IResult<&'a [u8], Vec<M30Sound>> {
    many0!(input, call!(m30_sound, hdr))
}

struct Sounds {
    key_sounds: Vec<audio::EffectStream>,
    bg_sounds: Vec<audio::EffectStream>,
}

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

pub fn dump_data<P: AsRef<Path>>(path: P) {
    let mut file_data = [0; 32];
    let mut file = File::open(path).expect("Failed to open ojn file");
    file.read_exact(&mut file_data);
    let hdr = header(&file_data);
    println!("{:?}", hdr);
}
