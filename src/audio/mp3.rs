use simplemad;
use audio::MusicStream;

use std::io;
use std::iter::Peekable;

use simplemad::{ Decoder, SimplemadError, MadFixed32 };

/// Lazy iterator over audio samples from an MP3
struct MP3Samples<R: io::Read + Send> {
    decoder: Peekable<simplemad::Decoder<R>>,
    current_samples: Option<Vec<Vec<MadFixed32>>>,
    current_samples_index: usize,

    /// What channel the next sample should come from
    current_channel: usize,
}

impl<R: io::Read + Send> MP3Samples<R> {
    fn new(decoder: Peekable<Decoder<R>>) -> MP3Samples<R> {
        MP3Samples {
            decoder: decoder,
            current_samples: None,
            current_samples_index: 0,
            current_channel: 0,
        }
    }
}

impl<R: io::Read + Send> Iterator for MP3Samples<R> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.current_samples.is_none() || self.current_samples_index == self.current_samples.as_ref().unwrap()[0].len() {
            loop {
                match self.decoder.next().unwrap() {
                    Ok(f) => {
                        self.current_samples = Some(f.samples);
                        self.current_samples_index = 0;
                        self.current_channel = 0;
                        break;
                    }

                    Err(SimplemadError::Mad(e)) => {
                        eprintln!("{:?}", e);
                    },

                    Err(SimplemadError::Read(e)) => {
                        eprintln!("{}", e);
                    },

                    Err(SimplemadError::EOF) => {
                        return None;
                    },
                }
            }
        }

        let sample = self.current_samples.as_ref().unwrap()[self.current_channel][self.current_samples_index].to_f32();
        self.current_channel = (self.current_channel + 1) % self.current_samples.as_ref().unwrap().len();
        if self.current_channel == 0 {
            self.current_samples_index += 1;
        }

        // TODO maybe not multiply?
        Some(sample * 5.0)
    }
}

// Hope nothing bad happens
unsafe impl<R: io::Read + Send> Send for MP3Samples<R> { }

/// Create a stream that reads from an mp3
pub fn decode<R: io::Read + Send + 'static>(reader: R) -> Result<MusicStream<f32>, String> {
    let mut decoder = match Decoder::decode(io::BufReader::new(reader)) {
        Ok(d) => d.peekable(),
        Err(e) => return Err(format!("{:?}", e)),
    };

    let sample_rate;
    let channel_count;

    {
        while let &Err(_) = decoder.peek().unwrap() { decoder.next(); }
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
