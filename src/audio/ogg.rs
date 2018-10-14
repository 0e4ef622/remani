//! Load OGG files that contain Vorbis (TODO add opus)

use crate::audio::GenericMusicStream;

use std::{io, vec};

use cpal::Sample;
use lewton::inside_ogg::OggStreamReader;

struct OggVorbisSamples<R: io::Read + io::Seek + Send + 'static> {
    ogg_stream_reader: OggStreamReader<R>,
    buffer: Option<vec::IntoIter<i16>>,
}

impl<R> Iterator for OggVorbisSamples<R>
where
    R: io::Read + io::Seek + Send + 'static
{
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.buffer.as_ref().map(|i| i.len() == 0).unwrap_or(true) {
            loop { // keep asking for another buffer until we get one that isn't empty
                match self.ogg_stream_reader.read_dec_packet_itl() {
                    Ok(o) => {
                        self.buffer = o
                            .map(|v| {
                                v.into_iter()
                            });
                        // check to make sure the buffer isn't empty
                        if self.buffer.as_ref().map(|v| v.len() != 0).unwrap_or(true) {
                            break;
                        }
                    }
                    Err(e) => { // janky error handling
                        remani_warn!("Error reading ogg: {}", e);
                        self.buffer = None;
                    }
                }
            }
        }
        self.buffer
            .as_mut()
            .and_then(|i| i.next().map(|n| n.to_f32()))
    }
}

pub(super) fn decode<R: io::Read + io::Seek + Send + 'static>(
    reader: R,
) -> Result<GenericMusicStream<impl Iterator<Item = f32> + Send, f32>, String> {

    let mut ogg_reader = lewton::inside_ogg::OggStreamReader::new(reader)
        .map_err(|e| format!("Failed to read ogg: {}", e))?;

    let channel_count = ogg_reader.ident_hdr.audio_channels;
    let sample_rate = ogg_reader.ident_hdr.audio_sample_rate;

    Ok(GenericMusicStream {
        samples: OggVorbisSamples {
            ogg_stream_reader: ogg_reader,
            buffer: None,
        },
        channel_count,
        sample_rate,
    })
}
