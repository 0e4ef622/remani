//! A basic music player purely for testing the audio module

use remani::audio;

use std::{ffi::OsStr, env, path, error::Error, thread};

fn output_help(binary_name: &OsStr) {
    println!("Usage:  {} path/to/music/file", binary_name.to_string_lossy());
}

fn main() -> Result<(), Box<dyn Error>> {
    let audio = audio::start_audio_thread(Default::default())?;
    let mut args = env::args_os();
    let binary = args.next().unwrap_or(format!("./{}", file!().rsplitn(2, ".rs").nth(1).unwrap()).into());
    let audio_filename = match args.next() {
        Some(s) => path::PathBuf::from(s),
        None => {
            output_help(&binary);
            return Ok(());
        }
    };

    let music = audio::music_from_path(audio_filename, audio.format())?;
    if audio.play_music(music).is_err() {
        Err("Error sending music to audio thread")?;
    }
    loop { thread::park(); }
}
