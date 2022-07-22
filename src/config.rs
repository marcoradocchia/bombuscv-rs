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

use crate::{args::Args, error::ErrorKind};
use directories::BaseDirs;
use serde::{de, Deserialize, Deserializer};
use std::{
    env,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    string::String,
};

/// Expands `~` in `path` to absolute HOME path.
pub fn expand_home(path: &Path) -> PathBuf {
    if let Ok(path) = path.strip_prefix("~") {
        // Generate the absolute path for HOME.
        BaseDirs::new()
            .expect("unable to find home directory")
            .home_dir()
            .to_path_buf()
            .join(path)
    } else {
        path.to_path_buf()
    }
}

/// Custom deserializer for `directory` field.
/// Automatically expands ~ and creates directory if doesn't exist.
fn deserialize_directory<'de, D>(directory: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path = expand_home(&PathBuf::deserialize(directory)?);
    if !path.is_dir() && fs::create_dir_all(&path).is_err() {
        return Err(de::Error::custom("unable to create specified directory"));
    }

    Ok(path)
}

/// Default value for /dev/video<index> capture camera index.
fn default_index() -> u8 {
    0
}

/// Default value for video capture frame height.
fn default_height() -> u16 {
    480
}

/// Default value for video capture frame width.
fn default_width() -> u16 {
    640
}

/// Default value for video capture framerate.
fn default_framerate() -> u8 {
    60
}

/// Default output video directory.
fn default_directory() -> PathBuf {
    // No base directories could be determined, so panicking is fine here.
    BaseDirs::new()
        .expect("unable to find HOME directory")
        .home_dir()
        .to_path_buf()
}

/// Default output video filename format, for example: `2022-06-23T11:49:00`.
fn default_format() -> String {
    String::from("%Y-%m-%dT%H:%M:%S")
}

/// Configuration options.
#[derive(Deserialize, Debug)]
pub struct Config {
    /// /dev/video<index> capture camera index.
    #[serde(default = "default_index")]
    pub index: u8,

    /// Video file as input.
    #[serde(skip_deserializing)]
    pub video: Option<PathBuf>,

    /// Video capture frame height.
    #[serde(default = "default_height")]
    pub height: u16,

    /// Video capture frame width.
    #[serde(default = "default_width")]
    pub width: u16,

    /// Video capture framerate.
    #[serde(default = "default_framerate")]
    pub framerate: u8,

    /// Output video directory.
    #[serde(
        default = "default_directory",
        deserialize_with = "deserialize_directory"
    )]
    pub directory: PathBuf,

    /// Output video filename format (see
    /// <https://docs.rs/chrono/latest/chrono/format/strftime/index.html> for valid specifiers).
    #[serde(default = "default_format")]
    pub format: String,

    /// Date&Time video overlay.
    #[serde(default)]
    pub overlay: bool,

    /// Disable colored output.
    #[serde(skip_deserializing, default)]
    pub no_color: bool,

    /// Mute standard output.
    #[serde(default)]
    pub quiet: bool,
}

/// Implement the Default trait for Config.
impl Default for Config {
    /// Default configuration.
    fn default() -> Self {
        Self {
            index: default_index(),
            video: None,
            height: default_height(),
            width: default_width(),
            framerate: default_framerate(),
            directory: default_directory(),
            format: default_format(),
            overlay: false,
            no_color: false,
            quiet: false,
        }
    }
}

impl Config {
    /// Parse configuration from config file, return Err on error.
    pub fn parse() -> Result<Self, ErrorKind> {
        if let Some(base_dirs) = BaseDirs::new() {
            // Fetch the environment variables for BOMBUSCV_CONFIG to hold a custom path to store
            // the configuration file.
            let config_file = if let Ok(path) = env::var("BOMBUSCV_CONFIG") {
                expand_home(Path::new(&path))
            } else {
                // XDG base spec directories: ~/.config/bomsucv/config.toml
                base_dirs
                    .config_dir()
                    .join(Path::new("bombuscv/config.toml"))
            };

            // On Err variant, return empty string (=> default config).
            let config = fs::read_to_string(config_file).unwrap_or_default();

            // Deserialize toml configuration file into Config.
            toml::from_str(&config).map_err(|e| ErrorKind::BrokenConfig(e.to_string()))
        } else {
            Err(ErrorKind::ConfigNotFound)
        }
    }

    /// Override configuration with command line arguments.
    pub fn override_with_args(mut self, args: Args) -> Self {
        if let Some(directory) = args.directory {
            self.directory = directory;
        }

        if let Some(format) = args.format {
            self.format = format;
        }

        if args.no_color {
            self.no_color = true;
        }

        if args.quiet {
            self.quiet = true;
        }

        // Input is video: disable overlay.
        if let Some(video) = args.video {
            self.video = Some(video);
            if self.overlay {
                self.overlay = false;
            }
        }

        if let Some(index) = args.index {
            self.index = index;
        }

        if let Some(height) = args.height {
            self.height = height;
        }

        if let Some(width) = args.width {
            self.width = width;
        }

        if let Some(framerate) = args.framerate {
            self.framerate = framerate;
        }

        if args.overlay {
            self.overlay = true;
        }

        self
    }
}
