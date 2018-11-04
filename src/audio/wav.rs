//! Load WAV files

use crate::audio::GenericMusicStream;

use std::io;

use cpal::Sample;
use either::Either;

pub(super) fn decode<R: io::Read + Send + 'static>(reader: R) -> Result<GenericMusicStream<impl Iterator<Item = f32>>, String> {
    let buf_reader = io::BufReader::new(reader);
    let wav_reader = hound::WavReader::new(buf_reader).map_err(|e| e.to_string())?;
    let format = wav_reader.spec();
    let channel_count = format.channels;
    let sample_rate = format.sample_rate;
    let samples = match format.sample_format {
        hound::SampleFormat::Int => {
            Either::Left(
                wav_reader
                .into_samples::<i16>()
                .map(|s| {
                    match s {
                        Ok(s) => s.to_f32(),
                        Err(e) => {
                            remani_warn!("wav read error: {}", e);
                            0.0
                        }
                    }
                })
            )
        }
        hound::SampleFormat::Float => {
            Either::Right(
                wav_reader
                .into_samples::<f32>()
                .map(|s| {
                    match s {
                        Ok(s) => s,
                        Err(e) => {
                            remani_warn!("wav read error: {}", e);
                            0.0
                        },
                    }
                })
            )
        }
    };

    Ok(GenericMusicStream {
        samples,
        channel_count: channel_count as u8,
        sample_rate,
    })
}
