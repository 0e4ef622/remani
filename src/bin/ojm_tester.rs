//! A binary for testing the OJN parser

use remani::chart::ojm_dump;

use std::{env, ffi::OsStr};

fn output_help(binary_name: &OsStr) {
    println!("Usage:  {} path/to/ojmfile", binary_name.to_string_lossy());
}

pub fn main() {
    let mut args = env::args_os();
    let binary = args.next().unwrap_or(format!("./{}", file!().rsplitn(2, ".rs").nth(1).unwrap()).into());
    let path = match args.next() {
        Some(s) => s,
        None => {
            output_help(&binary);
            return;
        }
    };
    ojm_dump(path);
}
