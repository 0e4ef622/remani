#![allow(unreachable_patterns)]

//! This module deals with audio playback.

#[cfg(feature = "mp3")]
mod mp3;
#[cfg(feature = "wav")]
mod wav;
#[cfg(feature = "ogg")]
mod ogg;

mod resample;

use std::{
    collections::VecDeque,
    error,
    fmt,
    iter::{self, Peekable},
    sync::mpsc,
    sync::Arc,
    thread,
    time,
};

use cpal::{self, Sample};

fn mix<I1, I2>(i1: I1, i2: I2) -> impl Iterator<Item = f32>
where
    I1: Iterator<Item = f32>,
    I2: Iterator<Item = f32>,
{
    let i1 = i1.map(|v| Some(v)).chain(iter::repeat(None));
    let i2 = i2.map(|v| Some(v)).chain(iter::repeat(None));

    // hand rolled variation of zip that stops at the end of the longest iterator as opposed to
    // the shortest iterator
    i1.zip(i2)
        .take_while(|&(o1, o2)| o1 != None || o2 != None)
        .map(|(o1, o2)| o1.unwrap_or(0.0) + o2.unwrap_or(0.0))
}

#[derive(Debug, Clone)]
pub struct EffectStream<S: cpal::Sample = f32> {
    samples: Arc<Vec<S>>,
    /// A number that the samples are multiplied by. i.e. 0.0 is 0% volume, 1.0 is 100% volume.
    volume: f32,
}

impl EffectStream {
    /// Mix two `EffectStream`s together into a new EffectStream
    pub fn mix(&self, other: &EffectStream) -> EffectStream {
        EffectStream {
            samples: Arc::new(
                mix(
                    self.samples.iter().cloned().map(|s| s*self.volume),
                    other.samples.iter().cloned().map(|s| s*other.volume),
                ).collect()
            ),
            volume: 1.0,
        }
    }
    /// Returns a zero length `EffectStream` for when you need an `EffectStream` but you don't
    /// actually have any sound data to put in it.
    pub fn empty() -> EffectStream {
        EffectStream {
            samples: Arc::new(vec![]),
            volume: 1.0,
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.set_volume(volume);
        self
    }

    pub fn volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

impl<S: cpal::Sample> From<(f32, Arc<Vec<S>>)> for EffectStream<S> {
    fn from(t: (f32, Arc<Vec<S>>)) -> Self {
        EffectStream {
            samples: t.1,
            volume: t.0,
        }
    }
}

impl<S: cpal::Sample> From<MusicStream<S>> for EffectStream<S> {
    fn from(a: MusicStream<S>) -> Self {
        EffectStream {
            samples: Arc::new(a.samples.collect()),
            volume: 1.0,
        }
    }
}

/// A struct that encapsulates a lazy iterator over audio samples with metadata.
pub struct MusicStream<S: cpal::Sample = f32> {
    /// An interleaved iterator of samples
    samples: Box<dyn Iterator<Item = S> + Send>,
    channel_count: u8,
    sample_rate: u32,
}

/// Gets converted into a MusicStream after resampling. Used to avoid
/// unnecessary extra dynamic dispatch.
struct GenericMusicStream<I: Iterator<Item = S> + Send, S: cpal::Sample = f32> {
    samples: I,
    channel_count: u8,
    sample_rate: u32,
}

impl<S: cpal::Sample> Iterator for MusicStream<S> {
    type Item = S;
    fn next(&mut self) -> Option<S> {
        self.samples.next()
    }
}

impl MusicStream {
    pub fn empty() -> Self {
        MusicStream {
            samples: Box::new(iter::repeat(0.0)),
            channel_count: 1,
            sample_rate: 1,
        }
    }
}

/// An iterator over a Vec contained in an Arc
#[derive(Debug)]
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

/// Contains information such as whether the audio thread is currently playing music or not.
#[derive(Debug, Clone, Copy)]
pub struct AudioStatus {
    pub is_playing_music: bool,
}

/// A handle to the audio thread that lets you send music and effects to play.
pub struct Audio<S: cpal::Sample = f32> {
    music_sender: mpsc::SyncSender<MusicStream<S>>,
    effect_sender: mpsc::SyncSender<(f32, ArcIter<S>)>,

    /// Used by `request_playhead()` to ask the audio thread to send the playback time to playhead_rcv.
    request_playhead_sender: mpsc::SyncSender<()>,

    /// Used by `request_status` to ask the audio thread to send it's status the next time it
    /// loops.
    request_status_sender: mpsc::SyncSender<()>,

    /// Gives the time of the samples that were just sent. (See `request_playhead`)
    playhead_rcv: mpsc::Receiver<(time::Instant, f64)>,

    /// Gives the status of the audio thread. (See `request_status` and `AudioStatus` for more
    /// information)
    status_rcv: mpsc::Receiver<AudioStatus>,

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
        let volume = effect.volume;
        self.effect_sender
            .try_send((volume, ArcIter::new(effect.samples)))
            .map_err(|e| {
                let s = match e {
                    mpsc::TrySendError::Full(m) => m,
                    mpsc::TrySendError::Disconnected(m) => m,
                }.1.inner();
                (volume, s).into()
            })
    }

    /// Sends a request to the audio thread for the current playhead of the music. `get_playhead`
    /// needs to be called afterwards to actually get the playhead.
    pub fn request_playhead(&self) -> Result<(), mpsc::TrySendError<()>> {
        self.request_playhead_sender.try_send(())
    }

    /// Get the audio thread's response to the request for the current playhead. Returns the time
    /// that the playhead was sent, and the playhead in seconds.
    pub fn get_playhead(&self) -> Option<(time::Instant, f64)> {
        self.playhead_rcv.try_recv().ok()
    }

    pub fn request_status(&self) -> Result<(), mpsc::TrySendError<()>> {
        self.request_status_sender.try_send(())
    }

    pub fn get_status(&self) -> Option<AudioStatus> {
        self.status_rcv.try_recv().ok()
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
pub fn start_audio_thread(mut audio_buffer_size: cpal::BufferSize) -> Result<Audio, AudioThreadError> {
    let device = cpal::default_output_device().ok_or(AudioThreadError::NoOutputDevice)?;

    println!("Using device {}", device.name());

    let format = device.default_output_format()?;

    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format, &mut audio_buffer_size)?;
    println!("Using audio buffer size {:?}", audio_buffer_size);
    event_loop.play_stream(stream_id.clone());

    let (request_playhead_tx, request_playhead_rx) = mpsc::sync_channel(1);
    let (request_status_tx, request_status_rx) = mpsc::sync_channel(1);
    let (send_playhead_tx, send_playhead_rx) = mpsc::sync_channel(4);
    let (send_status_tx, send_status_rx) = mpsc::sync_channel(4);
    let (music_tx, music_rx) = mpsc::sync_channel(2);
    let (effect_tx, effect_rx) = mpsc::sync_channel::<(f32, ArcIter<f32>)>(128);

    let channel_count = format.channels as usize;
    let sample_rate = format.sample_rate.0;

    // Spawn the audio thread
    thread::spawn(move || {
        // (f64, ArcIter<f32>) = (volume, effect_stream_samples_iterator)
        let mut effects: VecDeque<(f32, Peekable<ArcIter<f32>>)> = VecDeque::with_capacity(128);
        let mut music: Option<MusicStream> = None;

        // hopefully u64 is big enough and no one tries to play a 3 million year 192kHz audio file
        let mut current_music_frame_index: u64 = 0;

        // Audio loop
        event_loop.run(move |_, data| {
            while let Ok((volume, effect)) = effect_rx.try_recv() {
                effects.push_back((volume, effect.peekable()));
                // TODO do things
            }
            while let Ok(m) = music_rx.try_recv() {
                music = Some(m);
                current_music_frame_index = 0;
            }
            if request_playhead_rx.try_recv().is_ok() {
                send_playhead_tx.try_send((time::Instant::now(), current_music_frame_index as f64 / sample_rate as f64));
            }
            if request_status_rx.try_recv().is_ok() {
                send_status_tx.try_send(AudioStatus { is_playing_music: music.is_some() });
            }

            // Get samples and mix them
            // TODO use SIMD
            let mut s = |effects: &mut VecDeque<(f32, Peekable<ArcIter<f32>>)>| {
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
                for (volume, effect) in effects.iter_mut() {
                    if let Some(sample) = effect.next() {
                        s += sample.to_f32() * *volume;
                    }
                }
                s / 2.0 // TODO don't do this and maybe have a volume setting control this? ¯\_(ツ)_/¯
            };

            match data { // TODO vectorize?
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

            let mut i = 0;
            while i < effects.len() {
                if effects[i].1.peek().is_none() {
                    effects.swap_remove_back(i);
                } else {
                    i += 1;
                }
            }
        });
    });

    Ok(Audio {
        effect_sender: effect_tx,
        music_sender: music_tx,

        request_playhead_sender: request_playhead_tx,
        playhead_rcv: send_playhead_rx,
        request_status_sender: request_status_tx,
        status_rcv: send_status_rx,

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
    UnsupportedFormat(String),
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
            AudioLoadError::UnsupportedFormat(ref s) => write!(f, "Unsupported format: {}", s),
        }
    }
}

impl error::Error for AudioLoadError {
    fn description(&self) -> &str {
        match *self {
            AudioLoadError::Io(_) => "IO error",
            AudioLoadError::Decode(_) => "Decode error",
            AudioLoadError::UnsupportedFormat(_) => "Unsupported format",
        }
    }
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            AudioLoadError::Io(ref e) => Some(e),
            AudioLoadError::Decode(_) => None,
            AudioLoadError::UnsupportedFormat(_) => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MusicFormat {
    Mp3,
    Ogg,
    Wav,
}

impl From<MusicFormat> for &'static str {
    fn from(f: MusicFormat) -> Self {
        match f {
            MusicFormat::Mp3 => "mp3",
            MusicFormat::Ogg => "ogg",
            MusicFormat::Wav => "wav",
        }
    }
}

impl From<MusicFormat> for String {
    fn from(f: MusicFormat) -> Self {
        <&str>::from(f).into()
    }
}

fn maybe_resample<I>(
    stream: GenericMusicStream<I>,
    format: &cpal::Format
) -> MusicStream
where
    I: Iterator<Item = f32> + Send + 'static
{
    if stream.sample_rate == format.sample_rate.0 {
        MusicStream {
            samples: Box::new(stream.samples),
            channel_count: stream.channel_count,
            sample_rate: stream.sample_rate,
        }
    } else {
        resample::from_music_stream(stream, format.sample_rate.0)
    }
}

pub fn music_from_path<P: AsRef<Path>>(
    path: P,
    format: &cpal::Format,
) -> Result<MusicStream, AudioLoadError> {
    let file = File::open(&path)?;
    let extension = path
        .as_ref()
        .extension()
        .and_then(ffi::OsStr::to_str)
        .map(str::to_lowercase);

    let stream = match extension.as_ref().map(String::as_str) {
        #[cfg(feature = "mp3")]
        Some("mp3") => maybe_resample(mp3::decode(file).map_err(AudioLoadError::from)?, format),

        #[cfg(feature = "wav")]
        Some("wav") => maybe_resample(wav::decode(file).map_err(AudioLoadError::from)?, format),

        #[cfg(feature = "ogg")]
        Some("ogg") => maybe_resample(ogg::decode(file).map_err(AudioLoadError::from)?, format),

        Some(s) => return Err(AudioLoadError::UnsupportedFormat(s.into())),
        None => return Err(AudioLoadError::UnsupportedFormat("No extension".into())),
    };
    Ok(stream)
}

pub fn music_from_reader<R: io::Read + io::Seek + Send + 'static>(
    reader: R,
    cpal_format: &cpal::Format,
    music_format: MusicFormat,
) -> Result<MusicStream, AudioLoadError> {
    Ok(match music_format {
        #[cfg(feature = "mp3")]
        MusicFormat::Mp3 => maybe_resample(mp3::decode(reader).map_err(AudioLoadError::from)?, cpal_format),

        #[cfg(feature = "wav")]
        MusicFormat::Wav => maybe_resample(wav::decode(reader).map_err(AudioLoadError::from)?, cpal_format),

        #[cfg(feature = "ogg")]
        MusicFormat::Ogg => maybe_resample(ogg::decode(reader).map_err(AudioLoadError::from)?, cpal_format),

        f => return Err(AudioLoadError::UnsupportedFormat(f.into())),
    })
}
