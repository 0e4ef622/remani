//! O2Jam sound file parser module

use nom::*;

#[derive(Debug)]
struct OmcHeader {
    wav_count: i16,
    ogg_count: i16,
    wav_start: i32,
    ogg_start: i32,
}

named!(omc_header(&[u8]) -> OmcHeader,
    do_parse!(
        wav_count: le_i16 >>
        ogg_count: le_i16 >>
        wav_start: le_i32 >>
        ogg_start: le_i32 >>
        (OmcHeader { wav_count, ogg_count, wav_start, ogg_start })
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

#[derive(Debug)]
struct M30Header {
    file_format_version: i32,
    encryption_flag: M30Encryption,
    sample_count: i32,
    samples_offset: i32,
    payload_size: i32,
}

named!(m30_header(&[u8]) -> M30Header,
    do_parse!(
        file_format_version: le_i32 >>
        encryption_flag: m30_encryption >>
        sample_count: le_i32 >>
        samples_offset: le_i32 >>
        payload_size: le_i32 >>
        (M30Header { file_format_version, encryption_flag, sample_count, samples_offset, payload_size })
    )
);

enum Header {
    M30(M30Header),
    Omc(OmcHeader),
}

named!(header(&[u8]) -> Header,
    alt!(
        preceded!(alt!(tag!("OJM\0") | tag!("OMC\0")), omc_header) => { |h| Header::Omc(h) } |
        preceded!(tag!("M30\0"), m30_header) => { |h| Header::M30(h) }
    )
);
