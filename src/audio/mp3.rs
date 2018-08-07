//! Load MP3 files

use crate::audio::MusicStream;

use std::{io, iter::Peekable};

use simplemad::{self, Decoder, MadFixed32, SimplemadError};

/// Lazy iterator over audio samples from an MP3
struct MP3Samples<R: io::Read + Send> {
    decoder: Peekable<simplemad::Decoder<R>>,
    current_samples: Option<Vec<Vec<MadFixed32>>>,
    current_samples_index: usize,

    /// What channel the next sample should come from
    current_channel: usize,

    /// Whether the end of the file has been reached yet.
    eof: bool,
}

impl<R: io::Read + Send> MP3Samples<R> {
    fn new(decoder: Peekable<Decoder<R>>) -> MP3Samples<R> {
        MP3Samples {
            decoder: decoder,
            current_samples: None,
            current_samples_index: 0,
            current_channel: 0,
            eof: false,
        }
    }
}

impl<R: io::Read + Send> Iterator for MP3Samples<R> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.eof {
            return None;
        // If we need the next mp3 frame, get it
        } else if self.current_samples.is_none()
            || self.current_samples_index == self.current_samples.as_ref().unwrap()[0].len()
        {
            loop {
                match self.decoder.next() {
                    Some(r) => match r {
                        Ok(f) => {
                            self.current_samples = Some(f.samples);
                            self.current_samples_index = 0;
                            self.current_channel = 0;
                            break;
                        }
                        Err(SimplemadError::Mad(e)) => remani_warn!("libmad err: {:?}", e),
                        Err(SimplemadError::Read(e)) => remani_warn!("mp3 read err: {}", e),
                        Err(SimplemadError::EOF) => {
                            self.eof = true;
                            return None;
                        }
                    },
                    None => {
                        self.eof = true;
                        return None;
                    }
                }
            }
        }

        // This shouldn't ever error
        let current_samples = self
            .current_samples
            .as_ref()
            .expect("Something went terribly wrong in the mp3 module");

        let sample = current_samples[self.current_channel][self.current_samples_index].to_f32();
        self.current_channel = (self.current_channel + 1) % current_samples.len();
        if self.current_channel == 0 {
            self.current_samples_index += 1;
        }

        Some(sample)
    }
}

// Hope nothing bad happens
unsafe impl<R: io::Read + Send> Send for MP3Samples<R> {}

/// Create a stream that reads from an mp3
pub fn decode<R: io::Read + Send + 'static>(reader: R) -> Result<MusicStream<f32>, String> {
    let mut decoder = match Decoder::decode(reader) {
        Ok(d) => d.peekable(),
        Err(e) => return Err(format!("{:?}", e)),
    };

    let sample_rate;
    let channel_count;

    {
        // Get the sample rate and channel count.
        while let &Err(_) = decoder.peek().ok_or("Error finding audio metadata")? {
            decoder.next();
        }

        // This line should never panic
        let frame = decoder.peek().unwrap().as_ref().unwrap();

        sample_rate = frame.sample_rate;
        channel_count = frame.samples.len();
    }

    Ok(MusicStream {
        samples: Box::new(MP3Samples::new(decoder)),
        channel_count: channel_count as u8,
        sample_rate: sample_rate,
    })
}
