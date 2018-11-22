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
    filesize: i32,
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
        filesize: le_i32 >>
        (OmcHeader { encrypted, wav_count, ogg_count, wav_start, ogg_start, filesize })
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
    many0!(input, complete!(call!(m30_sound, hdr)))
}

#[derive(Debug)]
struct OmcWavSound {
    sound_name: String,
    format: i16,
    num_channels: i16,
    sample_rate: i32,
    bit_rate: i32,
    block_align: i16,
    bits_per_sample: i16,
    wav_data: Vec<u8>
}

/// Per file state for decrypting a WAV sound from the OMC format
#[derive(Debug, Copy, Clone)]
struct OmcWavDecryptState {
    acc_counter: i32,
    acc_keybyte: u8,
}

impl Default for OmcWavDecryptState {
    fn default() -> Self {
        OmcWavDecryptState {
            acc_keybyte: 0xFF,
            acc_counter: 0,
        }
    }
}

// Algorithm copied from
// https://github.com/open2jamorg/open2jam/blob/11384b3ca957828ae66a72c9e28edd42c97952d5/parsers/src/org/open2jam/parsers/OJMParser.java
fn omc_wav_decrypt(data: &[u8], state: &mut OmcWavDecryptState) -> Vec<u8> {
    const REARRANGE_TABLE: [u8; 290] = [
        0x10, 0x0E, 0x02, 0x09, 0x04, 0x00, 0x07, 0x01,
        0x06, 0x08, 0x0F, 0x0A, 0x05, 0x0C, 0x03, 0x0D,
        0x0B, 0x07, 0x02, 0x0A, 0x0B, 0x03, 0x05, 0x0D,
        0x08, 0x04, 0x00, 0x0C, 0x06, 0x0F, 0x0E, 0x10,
        0x01, 0x09, 0x0C, 0x0D, 0x03, 0x00, 0x06, 0x09,
        0x0A, 0x01, 0x07, 0x08, 0x10, 0x02, 0x0B, 0x0E,
        0x04, 0x0F, 0x05, 0x08, 0x03, 0x04, 0x0D, 0x06,
        0x05, 0x0B, 0x10, 0x02, 0x0C, 0x07, 0x09, 0x0A,
        0x0F, 0x0E, 0x00, 0x01, 0x0F, 0x02, 0x0C, 0x0D,
        0x00, 0x04, 0x01, 0x05, 0x07, 0x03, 0x09, 0x10,
        0x06, 0x0B, 0x0A, 0x08, 0x0E, 0x00, 0x04, 0x0B,
        0x10, 0x0F, 0x0D, 0x0C, 0x06, 0x05, 0x07, 0x01,
        0x02, 0x03, 0x08, 0x09, 0x0A, 0x0E, 0x03, 0x10,
        0x08, 0x07, 0x06, 0x09, 0x0E, 0x0D, 0x00, 0x0A,
        0x0B, 0x04, 0x05, 0x0C, 0x02, 0x01, 0x0F, 0x04,
        0x0E, 0x10, 0x0F, 0x05, 0x08, 0x07, 0x0B, 0x00,
        0x01, 0x06, 0x02, 0x0C, 0x09, 0x03, 0x0A, 0x0D,
        0x06, 0x0D, 0x0E, 0x07, 0x10, 0x0A, 0x0B, 0x00,
        0x01, 0x0C, 0x0F, 0x02, 0x03, 0x08, 0x09, 0x04,
        0x05, 0x0A, 0x0C, 0x00, 0x08, 0x09, 0x0D, 0x03,
        0x04, 0x05, 0x10, 0x0E, 0x0F, 0x01, 0x02, 0x0B,
        0x06, 0x07, 0x05, 0x06, 0x0C, 0x04, 0x0D, 0x0F,
        0x07, 0x0E, 0x08, 0x01, 0x09, 0x02, 0x10, 0x0A,
        0x0B, 0x00, 0x03, 0x0B, 0x0F, 0x04, 0x0E, 0x03,
        0x01, 0x00, 0x02, 0x0D, 0x0C, 0x06, 0x07, 0x05,
        0x10, 0x09, 0x08, 0x0A, 0x03, 0x02, 0x01, 0x00,
        0x04, 0x0C, 0x0D, 0x0B, 0x10, 0x05, 0x06, 0x0F,
        0x0E, 0x07, 0x09, 0x0A, 0x08, 0x09, 0x0A, 0x00,
        0x07, 0x08, 0x06, 0x10, 0x03, 0x04, 0x01, 0x02,
        0x05, 0x0B, 0x0E, 0x0F, 0x0D, 0x0C, 0x0A, 0x06,
        0x09, 0x0C, 0x0B, 0x10, 0x07, 0x08, 0x00, 0x0F,
        0x03, 0x01, 0x02, 0x05, 0x0D, 0x0E, 0x04, 0x0D,
        0x00, 0x01, 0x0E, 0x02, 0x03, 0x08, 0x0B, 0x07,
        0x0C, 0x09, 0x05, 0x0A, 0x0F, 0x04, 0x06, 0x10,
        0x01, 0x0E, 0x02, 0x03, 0x0D, 0x0B, 0x07, 0x00,
        0x08, 0x0C, 0x09, 0x06, 0x0F, 0x10, 0x05, 0x0A,
        0x04, 0x00,
    ];
    let length = data.len();
    let mut key = ((length % 17) << 4) + (length % 17);
    let block_size = length / 17;

    let mut decoded_data = vec![0; data.len()];
    for block_num in 0..17 {
        let block_start_encoded = block_size * block_num;
        let block_start_decoded = block_size * usize::from(REARRANGE_TABLE[key]);
        decoded_data[block_start_decoded..block_start_decoded+block_size].copy_from_slice(&data[block_start_encoded .. block_start_encoded+block_size]);
        key += 1;
    }

    for n in &mut decoded_data {
        let orig_byte = *n;
        if ((state.acc_keybyte << state.acc_counter) & 0x80) != 0 {
            *n = !*n;
        }
        state.acc_counter += 1;
        if state.acc_counter > 7 {
            state.acc_counter = 0;
            state.acc_keybyte = orig_byte;
        }
    }
    decoded_data
}

fn omc_wav_sound<'a>(input: &'a [u8], hdr: &OmcHeader, decrypt_state: &mut OmcWavDecryptState) -> IResult<&'a [u8], Option<OmcWavSound>> {
    do_parse!(input,
        sound_name: string_block!(32) >>
        format: le_i16 >>
        num_channels: le_i16 >>
        sample_rate: le_i32 >>
        bit_rate: le_i32 >>
        block_align: le_i16 >>
        bits_per_sample: le_i16 >>
        take!(4) >> // unk_data: i32
        chunk_size: le_i32 >>
        wav_data: take!(chunk_size) >>
        (OmcWavSound {
            sound_name,
            format,
            num_channels,
            sample_rate,
            bit_rate,
            block_align,
            bits_per_sample,
            wav_data: if hdr.encrypted {
                omc_wav_decrypt(wav_data, decrypt_state)
            } else {
                wav_data.to_owned()
            },
        })
    ).map(|(o, s)| if s.wav_data.is_empty() { (o, None) } else { (o, Some(s)) })
}

fn omc_wav_sounds<'a>(input: &'a [u8], hdr: &OmcHeader) -> IResult<&'a [u8], Vec<OmcWavSound>> {
    let mut decrypt_state = OmcWavDecryptState::default();
    many0!(input, complete!(call!(omc_wav_sound, hdr, &mut decrypt_state)))
        .map(|(o, v)| (o, v.into_iter().flatten().collect()))
}

struct OmcOggSound {
    sound_name: String,
    ogg_data: Vec<u8>,
}

/// Returns a Vec containing raw ogg data
fn omc_ogg_sound(input: &[u8]) -> IResult<&[u8], OmcOggSound> {
    do_parse!(input,
        sound_name: string_block!(32) >>
        ogg_data: length_bytes!(le_i32) >>
        (OmcOggSound {
            sound_name,
            ogg_data: ogg_data.to_owned(),
        })
    )
}

named!(omc_ogg_sounds(&[u8]) -> Vec<OmcOggSound>,
    many0!(complete!(omc_ogg_sound))
);

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
    let mut file = File::open(path).expect("Failed to open ojm file");
    file.read_exact(&mut file_data).expect("Failed to read ojm file");
    let (_, hdr) = header(&file_data).unwrap();
    println!("{:#?}", hdr);
    match hdr {
        Header::Omc(h) => {
            println!("Internal format: {}", if h.encrypted { "OMC" } else { "OJM" });
            let wav_section_len = h.ogg_start - h.wav_start;
            let ogg_section_len = h.filesize - h.ogg_start;
            let wav_section_len = wav_section_len as usize;
            let ogg_section_len = ogg_section_len as usize;
            let mut buffer = vec![0; wav_section_len.max(ogg_section_len)];
            let wav_sounds;
            let ogg_sounds;

            file.seek(SeekFrom::Start(h.wav_start as u64)).unwrap();
            file.read_exact(&mut buffer[0..wav_section_len]).unwrap();

            wav_sounds = omc_wav_sounds(&buffer[0..wav_section_len], &h).unwrap().1;

            file.seek(SeekFrom::Start(h.ogg_start as u64)).unwrap();
            file.read_exact(&mut buffer[0..ogg_section_len]).unwrap();

            ogg_sounds = omc_ogg_sounds(&buffer[0..ogg_section_len]).unwrap().1;

            println!("WAV sounds:");
            let mut i = 0;
            for sound in &wav_sounds {
                if i >= 4 {
                    break;
                }
                println!("{}", sound.sound_name);
                i += 1;
            }
            if wav_sounds.len() > 5 { println!("And {} others...", wav_sounds.len() - i - 1); }
            println!("OGG sounds:");
            i = 0;
            for sound in &ogg_sounds {
                if i >= 4 {
                    break;
                }
                println!("{}", sound.sound_name);
                i += 1;
            }
            if ogg_sounds.len() > 5 { println!("And {} others...", ogg_sounds.len() - i - 1); }
        }
        Header::M30(h) => {
            println!("Internal format: M30");
            let mut buffer = vec![];
            file.seek(SeekFrom::Start(h.samples_offset as u64)).unwrap();
            file.read_to_end(&mut buffer).unwrap();
            let sounds = m30_sounds(&buffer, &h).unwrap().1;
            println!("Sounds:");
            let mut i = 0;
            for sound in &sounds {
                if i >= 4 {
                    break;
                }
                println!("{}", sound.sound_name);
                i += 1;
            }
            if sounds.len() > 5 { println!("And {} others...", sounds.len() - i - 1); }
        }
    }
}
