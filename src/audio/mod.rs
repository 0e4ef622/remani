//! This module deals with audio playback.

#[cfg(feature = "mp3")]
mod mp3;

mod resample;

use std::{
    collections::VecDeque, error, fmt, iter::Peekable, sync::mpsc, sync::Arc, thread, time,
};

use cpal::{self, Sample};

#[derive(Debug, Clone)]
pub struct EffectStream<S: cpal::Sample>(Arc<Vec<S>>);

impl<S: cpal::Sample> From<Arc<Vec<S>>> for EffectStream<S> {
    fn from(a: Arc<Vec<S>>) -> Self {
        EffectStream::<S>(a)
    }
}

/// A struct that encapsulates a lazy iterator over audio samples with metadata.
pub struct MusicStream<S: cpal::Sample> {
    /// An interleaved iterator of samples
    samples: Box<dyn Iterator<Item = S> + Send>,
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
            inner,
            index: 0,
        }
    }
    pub fn inner(self) -> Arc<Vec<T>> {
        self.inner
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
            }
            None => None,
        }
    }
}

/// A handle to the audio thread that lets you send music and effects to play.
pub struct Audio<S: cpal::Sample> {
    music_sender: mpsc::SyncSender<MusicStream<S>>,
    effect_sender: mpsc::SyncSender<ArcIter<S>>,

    /// Used by `request_playhead()` to ask the audio thread to send the playback time to playhead_rcv.
    request_playhead_sender: mpsc::SyncSender<()>,

    /// Gives the time of the samples that were just sent. (See request_playhead)
    playhead_rcv: mpsc::Receiver<(time::Instant, f64)>,

    /// The format of the audio device being used
    format: cpal::Format,
}

impl<S: cpal::Sample> Audio<S> {
    /// Start playing music, returning the passed in music stream if there was an error
    pub fn play_music(&self, music: MusicStream<S>) -> Result<(), MusicStream<S>> {
        self.music_sender.try_send(music).or_else(|e| match e {
            mpsc::TrySendError::Full(m) => Err(m),
            mpsc::TrySendError::Disconnected(m) => Err(m),
        })
    }

    /// Play a sound effect/hitsound, returning the passed in effect stream if there was an error
    pub fn play_effect(&self, effect: EffectStream<S>) -> Result<(), EffectStream<S>> {
        self.effect_sender
            .try_send(ArcIter::new(effect.0))
            .or_else(|e| {
                Err(match e {
                    mpsc::TrySendError::Full(m) => m,
                    mpsc::TrySendError::Disconnected(m) => m,
                }.inner()
                .into())
            })
    }

    /// Sends a request to the audio thread for the current playhead of the music.
    pub fn request_playhead(&self) -> Result<(), mpsc::TrySendError<()>> {
        self.request_playhead_sender.try_send(())
    }

    /// Get the audio thread's response to the request for the current playhead. Returns the time
    /// that the playhead was sent, and the playhead in seconds.
    pub fn get_playhead(&self) -> Option<(time::Instant, f64)> {
        self.playhead_rcv.try_recv().ok()
    }

    pub fn format(&self) -> &cpal::Format {
        &self.format
    }
}

#[derive(Debug)]
pub enum AudioThreadError {
    NoOutputDevice,
    DefaultFormatError(cpal::DefaultFormatError),
    OutputStreamCreationError(cpal::CreationError),
}

impl From<cpal::DefaultFormatError> for AudioThreadError {
    fn from(e: cpal::DefaultFormatError) -> Self {
        AudioThreadError::DefaultFormatError(e)
    }
}

impl From<cpal::CreationError> for AudioThreadError {
    fn from(e: cpal::CreationError) -> Self {
        AudioThreadError::OutputStreamCreationError(e)
    }
}

impl fmt::Display for AudioThreadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            AudioThreadError::NoOutputDevice => write!(f, "No output device found"),
            AudioThreadError::DefaultFormatError(_) => write!(f, "Error requesting stream format"),
            AudioThreadError::OutputStreamCreationError(ref e) => {
                write!(f, "Error building audio stream: {}", e)
            }
        }
    }
}

impl error::Error for AudioThreadError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            AudioThreadError::NoOutputDevice => None,
            AudioThreadError::DefaultFormatError(_) => None,
            AudioThreadError::OutputStreamCreationError(ref e) => Some(e),
        }
    }
    fn description(&self) -> &str {
        match *self {
            AudioThreadError::NoOutputDevice => "No output device found",
            AudioThreadError::DefaultFormatError(_) => "Error requesting stream format",
            AudioThreadError::OutputStreamCreationError(_) => "Error building audio stream",
        }
    }
}

/// Starts the audio thread and returns an object that can be used to communicate with the audio
/// thread.
pub fn start_audio_thread(mut audio_buffer_size: cpal::BufferSize) -> Result<Audio<f32>, AudioThreadError> {
    let device = cpal::default_output_device().ok_or(AudioThreadError::NoOutputDevice)?;

    println!("Using device {}", device.name());

    let format = device.default_output_format()?;

    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format, &mut audio_buffer_size)?;
    println!("Using audio buffer size {:?}", audio_buffer_size);
    event_loop.play_stream(stream_id.clone());

    let (request_playhead_tx, request_playhead_rx) = mpsc::sync_channel(1);
    let (send_playhead_tx, send_playhead_rx) = mpsc::sync_channel(4);
    let (music_tx, music_rx) = mpsc::sync_channel(2);
    let (effect_tx, effect_rx) = mpsc::sync_channel::<ArcIter<f32>>(128);

    let channel_count = format.channels as usize;
    let sample_rate = format.sample_rate.0;

    // Spawn the audio thread
    thread::spawn(move || {
        let mut effects: VecDeque<Peekable<ArcIter<f32>>> = VecDeque::with_capacity(128);
        let mut music: Option<MusicStream<f32>> = None;

        // hopefully u64 is big enough and no one tries to play a 3 million year 192kHz audio file
        let mut current_music_frame_index: u64 = 0;

        // Audio loop
        event_loop.run(move |_, data| {
            while let Ok(effect) = effect_rx.try_recv() {
                effects.push_back(effect.peekable());
                // TODO do things
            }
            while let Ok(m) = music_rx.try_recv() {
                music = Some(m);
                current_music_frame_index = 0;
            }
            if request_playhead_rx.try_recv().is_ok() {
                send_playhead_tx.try_send((time::Instant::now(), current_music_frame_index as f64 / sample_rate as f64));
            }

            // Get samples and mix them
            let mut s = |effects: &mut VecDeque<Peekable<ArcIter<f32>>>| {
                let mut s = match music {
                    Some(ref mut m) => match m.next() {
                        Some(n) => n,
                        None => {
                            music = None;
                            0.0
                        }
                    }
                    None => 0.0
                };
                for effect in effects.iter_mut() {
                    if let Some(sample) = effect.next() {
                        s += sample.to_f32();
                    }
                }
                s / 2.0 // TODO don't do this and maybe have a volume setting control this? ¯\_(ツ)_/¯
            };

            match data {
                cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                    for frame in buffer.chunks_mut(channel_count) {
                        for sample in frame.iter_mut() {
                            *sample = s(&mut effects).to_u16();
                        }
                        current_music_frame_index += 1;
                    }
                },

                cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                    for frame in buffer.chunks_mut(channel_count) {
                        for sample in frame.iter_mut() {
                            *sample = s(&mut effects).to_i16();
                        }
                        current_music_frame_index += 1;
                    }
                },

                cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                    for frame in buffer.chunks_mut(channel_count) {
                        for sample in frame.iter_mut() {
                            *sample = s(&mut effects);
                        }
                        current_music_frame_index += 1;
                    }
                },
                _ => (),
            }

            while let Some(effect) = effects.front_mut() {
                if effect.peek().is_none() {
                    effects.pop_front();
                } else {
                    break;
                }
            }
        });
    });

    Ok(Audio {
        effect_sender: effect_tx,
        music_sender: music_tx,

        request_playhead_sender: request_playhead_tx,
        playhead_rcv: send_playhead_rx,

        format,
    })
}

use std::ffi;
use std::fs::File;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub enum AudioLoadError {
    Io(io::Error),
    Decode(String),
}

impl From<io::Error> for AudioLoadError {
    fn from(e: io::Error) -> Self {
        AudioLoadError::Io(e)
    }
}

impl From<String> for AudioLoadError {
    fn from(s: String) -> Self {
        AudioLoadError::Decode(s)
    }
}

impl fmt::Display for AudioLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            AudioLoadError::Io(ref e) => write!(f, "IO error: {}", e),
            AudioLoadError::Decode(ref s) => write!(f, "Decode error: {}", s),
        }
    }
}

impl error::Error for AudioLoadError {
    fn description(&self) -> &str {
        match *self {
            AudioLoadError::Io(_) => "IO error",
            AudioLoadError::Decode(_) => "Decode error",
        }
    }
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            AudioLoadError::Io(ref e) => Some(e),
            AudioLoadError::Decode(_) => None,
        }
    }
}

pub fn music_from_path<P: AsRef<Path>>(
    path: P,
    format: &cpal::Format,
) -> Result<MusicStream<f32>, AudioLoadError> {
    let file = File::open(&path)?;
    let extension = path
        .as_ref()
        .extension()
        .and_then(ffi::OsStr::to_str)
        .map(str::to_lowercase);

    match extension.as_ref().map(String::as_str) {
        #[cfg(feature = "mp3")]
        Some("mp3") => {
            let stream = mp3::decode(file).map_err(AudioLoadError::from)?;
            if stream.sample_rate == format.sample_rate.0 {
                Ok(stream)
            } else {
                Ok(resample::from_music_stream(stream, format.sample_rate.0))
            }
        }

        _ => panic!("Unsupported format"),
    }
}
