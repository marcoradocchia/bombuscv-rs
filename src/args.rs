// bombuscv: OpenCV based motion detection/recording software built for research on bumblebees.
// Copyright (C) 2022 Marco Radocchia
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see https://www.gnu.org/licenses/.

use crate::config::expand_home;
use clap::ArgAction::{Set, SetTrue};
pub use clap::Parser;
use std::{fs, path::PathBuf};

/// Custom parser for `directory` field.
/// Automatically expands ~ and creates directory if doesn't exist.
pub fn parse_directory(directory: &str) -> Result<PathBuf, String> {
    let path = expand_home(&PathBuf::from(directory));
    if !path.is_dir() && fs::create_dir_all(&path).is_err() {
        return Err(String::from("unable to create specified directory"));
    }

    Ok(path)
}

/// Custom parser for `video` field.
fn parse_video(video: &str) -> Result<PathBuf, String> {
    let video = expand_home(&PathBuf::from(video));
    match video.is_file() {
        true => Ok(video),
        false => Err(String::from("the given path is not a file")),
    }
}

/// OpenCV motion detection/video-recording tool developed for research on Bumblebees.
#[derive(Parser, Debug)]
#[clap(
    author = "Marco Radocchia <marco.radocchia@outlook.com>",
    version,
    about,
    long_about = None
)]
pub struct Args {
    /// /dev/video<INDEX> capture camera index.
    #[clap(short, long, action = Set)]
    pub index: Option<u8>,

    /// Video file as input.
    #[clap(
        short,
        long,
        value_parser = parse_video,
        conflicts_with_all = &["index", "overlay", "height", "width", "framerate"]
    )]
    pub video: Option<PathBuf>,

    /// Video capture frame height.
    #[clap(short = 'H', long, action = Set)]
    pub height: Option<u16>,

    /// Video capture frame width.
    #[clap(short = 'W', long, action = Set)]
    pub width: Option<u16>,

    /// Video capture framerate.
    #[clap(short, long, action = Set)]
    pub framerate: Option<u8>,

    /// Output video directory.
    #[clap(short, long, value_parser = parse_directory)]
    pub directory: Option<PathBuf>,

    /// Output video filename format (see
    /// <https://docs.rs/chrono/latest/chrono/format/strftime/index.html> for valid specifiers).
    #[clap(long, action = Set)]
    pub format: Option<String>,

    /// Date&Time video overlay.
    #[clap(short, long, action = SetTrue)]
    pub overlay: bool,

    /// Disable colored output.
    #[clap(long, action = SetTrue)]
    pub no_color: bool,

    /// Mute standard output.
    #[clap(short, long, action = SetTrue)]
    pub quiet: bool,
}
