//! This module deals with audio playback.

mod mp3;

use std::sync::mpsc;
use std::collections::VecDeque;
use std::iter;
use std::iter::Peekable;
use std::sync::Arc;
use std::thread;

//use sample;
use cpal;
use cpal::Sample;

//pub type MusicStream<S: cpal::Sample> = Box<Iterator<Item = S> + Send>;
pub type EffectStream<S: cpal::Sample> = Arc<Vec<S>>;

/// A lazy iterator over audio samples
pub struct MusicStream<S: cpal::Sample> {
    samples: Box<Iterator<Item = S> + Send>,
    channel_count: u8,
    sample_rate: u32,
}

impl<S: cpal::Sample> Iterator for MusicStream<S> {
    type Item = S;
    fn next(&mut self) -> Option<S> {
        self.samples.next()
    }
}

/// An iterator over a Vec contained in an Arc
pub struct ArcIter<T: Copy> {
    inner: Arc<Vec<T>>,
    index: usize,
}

impl<T: Copy> ArcIter<T> {
    pub fn new(inner: Arc<Vec<T>>) -> ArcIter<T> {
        ArcIter {
            inner: inner,
            index: 0,
        }
    }
}

impl<T: Copy> Iterator for ArcIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let v = self.inner.get(self.index);
        match v {
            Some(&n) => {
                self.index += 1;
                Some(n)
            },
            None => None,
        }
    }
}

/// A handle to the audio thread that lets you send music and effects to play
pub struct Audio<S: cpal::Sample> {
    music_sender: mpsc::SyncSender<MusicStream<S>>,
    effect_sender: mpsc::SyncSender<ArcIter<S>>,

    /// Ask the audio thread to start streaming the playback time to playhead_rcv
    //request_playhead: mpsc::SyncSender<bool>,

    /// Gives the time of the samples that were just sent.
    //playhead_rcv: mpsc::Receiver<f64>,

    format: cpal::Format,
}

impl<S: cpal::Sample> Audio<S> {

    /// Start playing music
    pub fn play_music(&self, music: MusicStream<S>) -> Result<(), mpsc::TrySendError<MusicStream<S>>> {
        self.music_sender.try_send(music)
    }

    /// Play a sound effect/hitsound
    pub fn play_effect(&self, effect: EffectStream<S>) -> Result<(), mpsc::TrySendError<ArcIter<S>>> {
        self.effect_sender.try_send(ArcIter::new(effect))
    }

    pub fn format(&self) -> &cpal::Format { &self.format }
}

pub fn start_audio_thread() -> Audio<f32> {

    let device = cpal::default_output_device().expect("Failed to get default output device");

    println!("Using device {}", device.name());

    let format = device.default_output_format().expect("Failed to get default output format");

    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id.clone());

    let (music_tx, music_rx) = mpsc::sync_channel::<MusicStream<f32>>(2);
    let (effect_tx, effect_rx) = mpsc::sync_channel::<ArcIter<f32>>(128);

    thread::spawn(move || {

        let mut effects: VecDeque<Peekable<ArcIter<f32>>> = VecDeque::with_capacity(128);
        let mut music: MusicStream<f32> = MusicStream {
            samples: Box::new(iter::empty::<f32>()),
            channel_count: 0,
            sample_rate: 44100,
        };

        event_loop.run(move |_, data| {
            while let Ok(effect) = effect_rx.try_recv() {
                effects.push_back(effect.peekable());
                // do things
            }
            while let Ok(m) = music_rx.try_recv() {
                music = m;
            }

            let mut s = |effects: &mut VecDeque<Peekable<ArcIter<f32>>>| {
                let mut s = match music.next() {
                    None => 0.0,
                    Some(sample) => sample.to_f32(),
                };
                for effect in effects.iter_mut() {
                    if let Some(sample) = effect.next() {
                        s += sample.to_f32();
                    }
                }
                s / 2.0
            };

            match data {
                cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                    for sample in buffer.iter_mut() {
                        *sample = s(&mut effects).to_u16();
                    }
                },

                cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                    for sample in buffer.iter_mut() {
                        *sample = s(&mut effects).to_i16();
                    }
                },

                cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                    for sample in buffer.iter_mut() {
                        *sample = s(&mut effects).to_f32();;
                    }
                },
                _ => (),
            }
            while effects.front_mut().is_some() {
                if effects.front_mut().unwrap().peek().is_none() {
                    effects.pop_front();
                } else {
                    break;
                }
            }
        });
    });

    Audio {
        effect_sender: effect_tx,
        music_sender: music_tx,
        format: format,
    }
}

use std::fs::File;
use std::path::Path;
use std::ffi;
use std::io;

pub fn music_from_path<P: AsRef<Path>>(path: P, format: &cpal::Format) -> MusicStream<f32> {

    let file = File::open(&path).expect("Audio file not found");

    match path.as_ref().extension().and_then(ffi::OsStr::to_str) {

        Some("mp3") => mp3::decode(file).unwrap(),

        _ => panic!("Unsupported format"),
    }
}
