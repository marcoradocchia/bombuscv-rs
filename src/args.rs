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
pub use clap::Parser;
use std::path::PathBuf;

/// Validate framerate CLI argument.
pub fn validate_framerate(framerate: &str) -> Result<(), String> {
    let err_msg = || String::from("the framerate must be a positive floating point number.");
    match framerate.parse::<f64>() {
        Ok(framerate) => {
            if framerate < 0. {
                return Err(err_msg());
            }
            Ok(())
        }
        Err(_) => Err(err_msg()),
    }
}

/// Validate output video directory.
pub fn validate_directory(directory: &str) -> Result<(), String> {
    match expand_home(&PathBuf::from(directory)).is_dir() {
        true => Ok(()),
        false => Err(String::from("the given path is not a directory")),
    }
}

/// Validate input video path.
fn validate_video(video: &str) -> Result<(), String> {
    match expand_home(&PathBuf::from(video)).is_file() {
        true => Ok(()),
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
    #[clap(short, long)]
    pub index: Option<u8>,

    /// Video file as input.
    #[clap(short, long, validator = validate_video, conflicts_with_all = &["index", "overlay", "framerate", "resolution"])]
    pub video: Option<PathBuf>,

    /// Video framerate.
    #[clap(short, long, validator = validate_framerate)]
    pub framerate: Option<f64>,

    /// Video resolution (standard 16:9 formats).
    #[clap(
        short,
        long,
        possible_values = ["480p", "576p", "720p", "768p", "900p", "1080p", "1440p", "2160p"]
    )]
    pub resolution: Option<String>,

    /// Output video directory.
    #[clap(short, long, validator = validate_directory)]
    pub directory: Option<PathBuf>,

    /// Output video filename format (see
    /// <https://docs.rs/chrono/latest/chrono/format/strftime/index.html> for valid specifiers).
    #[clap(long)]
    pub format: Option<String>,

    /// Enable Date&Time video overlay.
    #[clap(short, long)]
    pub overlay: bool,

    /// Mute standard output.
    #[clap(short, long)]
    pub quiet: bool,
}
