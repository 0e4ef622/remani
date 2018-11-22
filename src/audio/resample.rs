use crate::audio::{GenericMusicStream, MusicStream};
use std::iter::Peekable;

// TODO maybe use higher quality resampling algorithm?

/// Resample a MusicStream using linear interpolation
pub struct Resample<I: Iterator<Item = f32> + Send> {
    /// The iterator that yields interleaved audio samples
    /// (e.g. an iterator for an audio stream with 3 channels would
    /// yield samples for channel 1, then 2, then 3, then 1, then 2, ...
    samples: Peekable<I>,

    channel_count: usize,
    from_sample_rate: u32,
    to_sample_rate: u32,
    /// What channel the next sample from the iterator corresponds with.
    channel_offset: usize,

    /// Some number between 0 and `to_sample_rate`.
    ///
    /// This is divided by `to_sample_rate` to calculate the coefficient for
    /// linear interpolation.
    sampling_offset: u32,

    /// Holds the previous audio frame (1 sample from each channel).
    previous_values: Vec<f32>,

    /// Holds the next audio frame (1 sample from each channel).
    next_values: Vec<f32>,
}

impl<I: Iterator<Item = f32> + Send> Iterator for Resample<I> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let return_value = if self.previous_values.len() < self.channel_count && self.next_values.len() < self.channel_count {
            let next_sample = match self.samples.next() {
                Some(s) => s,
                None => return None,
            };
            let next_next_sample = match self.samples.peek() {
                Some(s) => *s,
                None => return None,
            };
            self.previous_values.push(next_sample);
            self.next_values.push(next_next_sample);
            Some(next_sample)
        } else {
            if self.channel_offset == 0 {
                self.sampling_offset += self.from_sample_rate;
                while self.sampling_offset >= self.to_sample_rate {
                    self.sampling_offset -= self.to_sample_rate;
                    for n in 0..self.channel_count {
                        self.previous_values[n] = self.next_values[n];
                        let next_sample = match self.samples.next() {
                            Some(s) => s,
                            None => return None,
                        };
                        self.next_values[n] = next_sample;
                    }
                }
            }
            let prev_sample = self.previous_values[self.channel_offset];
            let next_sample = self.next_values[self.channel_offset];
            Some(prev_sample
                 + (next_sample - prev_sample)
                 * self.sampling_offset as f32 / self.to_sample_rate as f32)
        };
        self.channel_offset += 1;
        if self.channel_offset >= self.channel_count {
            self.channel_offset = 0;
        }
        return_value
    }
}

pub(super) fn from_music_stream<I>(
    stream: GenericMusicStream<I, f32>,
    target_sample_rate: u32,
) -> MusicStream
where
    I: Iterator<Item = f32> + Send + 'static
{
    let le_samples = Resample {
        samples: stream.samples.peekable(),
        channel_count: stream.channel_count as usize,
        from_sample_rate: stream.sample_rate,
        to_sample_rate: target_sample_rate,
        channel_offset: 0,
        sampling_offset: 0,
        previous_values: Vec::new(),
        next_values: Vec::new(),
    };

    MusicStream {
        samples: Box::new(le_samples),
    }
}
