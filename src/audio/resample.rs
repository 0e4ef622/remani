use audio::MusicStream;
use cpal;
use std::iter::Peekable;

// TODO maybe use higher quality resampling algorithm?

/// Resample a MusicStream using linear interpolation
pub struct Resample<S: cpal::Sample> {
    samples: Peekable<Box<dyn Iterator<Item=S> + Send>>,
    num_channels: usize,
    from_sample_rate: u32,
    to_sample_rate: u32,
    channel_offset: usize,
    sampling_offset: u32,
    previous_values: Vec<S>,
    next_values: Vec<S>,
}

impl<S: cpal::Sample> Iterator for Resample<S> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let return_value;
        if self.previous_values.len() < self.num_channels && self.next_values.len() < self.num_channels {
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
            return_value = Some(next_sample.to_f32());
        } else {
            if self.channel_offset == 0 {
                // TODO this overflows when the end of the audio is reached, although this isn't illegal.
                // Need to edit audio/mod.rs to stop using an iterator once it's been used up, or fuse it.
                self.sampling_offset += self.from_sample_rate;
                while self.sampling_offset >= self.to_sample_rate {
                    self.sampling_offset -= self.to_sample_rate;
                    for n in 0..self.num_channels {
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
            return_value = Some(prev_sample.to_f32() + (next_sample.to_f32() - prev_sample.to_f32()) * self.sampling_offset as f32 / self.to_sample_rate as f32);
        }
        self.channel_offset += 1;
        if self.channel_offset >= self.num_channels {
            self.channel_offset = 0;
        }
        return_value
    }
}

pub fn from_music_stream<S: cpal::Sample + Send + 'static>(stream: MusicStream<S>, target_sample_rate: u32) -> MusicStream<f32> {
    let le_samples = Resample {
        samples: stream.samples.peekable(),
        num_channels: stream.channel_count as usize,
        from_sample_rate: stream.sample_rate,
        to_sample_rate: target_sample_rate,
        channel_offset: 0,
        sampling_offset: 0,
        previous_values: Vec::new(),
        next_values: Vec::new(),
    };

    MusicStream {
        samples: Box::new(le_samples),
        channel_count: stream.channel_count,
        sample_rate: target_sample_rate,
    }
}
