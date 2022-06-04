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

use crate::args::Args;
use directories::BaseDirs;
use serde::Deserialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};
use validator::{Validate, ValidationError};

/// Expands `~` in `path` to absolute HOME path.
pub fn expand_home(path: &Path) -> PathBuf {
    match path.strip_prefix("~") {
        // `~` found: replace it with the absolute HOME path.
        Ok(path) => {
            // Generate the absolute path for HOME.
            let home = match BaseDirs::new() {
                Some(base_dirs) => base_dirs.home_dir().to_path_buf(),
                None => {
                    eprintln!("error: unable to find home directory");
                    process::exit(1);
                }
            };
            // Insert the absolute HOME path at the beginning of the path.
            home.join(path)
        }
        // `~` not found: return the given path as is
        Err(_) => path.to_path_buf(),
    }
}

/// Validate video resolution config option.
fn validate_resolution(resolution: &str) -> Result<(), ValidationError> {
    let valid_resolutions = [
        "480p", "576p", "720p", "768p", "900p", "1080p", "1440p", "2160p",
    ];

    match valid_resolutions.contains(&resolution) {
        true => Ok(()),
        false => Err(ValidationError::new("possible_value")),
    }
}

/// Validate output video directory path.
fn validate_directory(path: &Path) -> Result<(), ValidationError> {
    match expand_home(path).is_dir() {
        true => Ok(()),
        false => Err(ValidationError::new("path")),
    }
}

/// Validate input video path.
fn validate_video(path: &Path) -> Result<(), ValidationError> {
    match expand_home(path).is_file() {
        true => Ok(()),
        false => Err(ValidationError::new("path")),
    }
}

/// Default value for /dev/video<index> capture camera index.
fn default_index() -> u8 {
    0
}

/// Default value for video framerate.
fn default_framerate() -> f64 {
    60.
}

/// Dafault value for video resolution.
fn default_resolution() -> String {
    String::from("480p")
}

/// Default output video directory.
fn default_directory() -> PathBuf {
    match BaseDirs::new() {
        Some(base_dirs) => base_dirs.home_dir().to_path_buf(),
        None => {
            eprintln!("error: unable to find home directory");
            process::exit(1);
        }
    }
}

/// Default output video filename format.
fn default_format() -> String {
    String::from("%Y-%m-%dT%H:%M:%S")
}

/// Configuration options.
#[derive(Deserialize, Validate, Debug)]
pub struct Config {
    /// /dev/video<index> capture camera index.
    #[serde(default = "default_index")]
    pub index: u8,

    /// Video framerate.
    #[validate(range(min = 1.0, message = "invalid framerate (must be >1.0)"))]
    #[serde(default = "default_framerate")]
    pub framerate: f64,

    /// Video resolution (standard 16:9 formats).
    #[validate(custom(function = "validate_resolution", message = "invalid resolution value"))]
    #[serde(default = "default_resolution")]
    pub resolution: String,

    /// Output video directory.
    #[validate(custom(
        function = "validate_directory",
        message = "given path is not a directory"
    ))]
    #[serde(default = "default_directory")]
    pub directory: PathBuf,

    /// Input video file.
    #[validate(custom(
        function = "validate_video",
        message = "given path is not a video file"
    ))]
    #[serde(default)]
    pub video: Option<PathBuf>,

    /// Output video filename format (see https://docs.rs/chrono/latest/chrono/format/strftime/index.html for valid specifiers).
    #[serde(default = "default_format")]
    pub format: String,

    /// Enable Date/Time video overlay.
    #[serde(default)]
    pub overlay: bool,

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
            framerate: default_framerate(),
            resolution: default_resolution(),
            directory: default_directory(),
            video: None,
            format: default_format(),
            overlay: false,
            quiet: false,
        }
    }
}

impl Config {
    /// Parse configuration from config file.
    pub fn parse() -> Self {
        let mut config = if let Some(base_dirs) = BaseDirs::new() {
            // Below the OS specific config dir values.
            // Lin: /home/alice/.config/
            // Mac: /Users/Alice/Library/Application Support/
            // Win: C:\Users\Alice\AppData\Roaming\

            // Fetch the environment variables for BOMBUSCV_CONFIG to hold a custom path to store
            // the configuration file.
            let config_file = fs::read_to_string(match env::var("BOMBUSCV_CONFIG") {
                // BOMBUSCV_CONFIG env variable set, so expand home and use it as the config
                Ok(path) => expand_home(Path::new(&path)),
                // BOMBUSCV_CONFIG env variable unset or invalid: use default config file location.
                Err(_) => base_dirs
                    .config_dir()
                    .join(Path::new("bombuscv/config.toml")),
            })
            .unwrap_or_default();

            // Parse toml configuration file.
            let config: Config = match toml::from_str(&config_file) {
                Err(e) => {
                    eprintln!("error: invalid config '{e}', using defaults");
                    Config::default()
                }
                Ok(config) => config,
            };

            // Values passed the parsing, now validate config values.
            match config.validate() {
                // Values pass validation, return parsed configuration.
                Ok(_) => config,
                // Values don't pass validation, return default configuration and warn the user.
                Err(errors) => {
                    // TODO: not very elegant
                    // Gather all the invalid value into a sting and display it as an error
                    // message.
                    let mut error_msg =
                        String::from("error: invalid configuration options, using defaults\n");
                    for err in errors.field_errors() {
                        let msg = &err.1.first().unwrap().message;
                        error_msg.push_str(&format!("-> {}: {}\n", err.0, msg.as_ref().unwrap()));
                    }
                    error_msg.pop(); // remove last new line
                    eprintln!("{}", error_msg);

                    Config::default()
                }
            }
        } else {
            eprintln!("warning: no valid config path found on the system, using defaults");
            Config::default()
        };

        // If video path is given using `~` as HOME directory, expand to absolute path.
        match config.video {
            Some(video) => {
                config.video = Some(expand_home(&video));
                if config.overlay {
                    config.overlay = false;
                    eprintln!("warning: ignoring `overlay` option while using `video` option in configuration file.");
                }
            }
            None => config.video = None,
        }

        // If video directory is given using ~ as home directory, expand to absolute path.
        config.directory = expand_home(&config.directory);

        config
    }

    /// Override configuration with command line arguments.
    pub fn override_with_args(mut self, args: Args) -> Self {
        if let Some(index) = args.index {
            self.index = index;
        }
        if let Some(framerate) = args.framerate {
            self.framerate = framerate;
        }
        if let Some(resolution) = args.resolution {
            self.resolution = resolution;
        }
        if let Some(directory) = args.directory {
            self.directory = directory;
        };
        if let Some(video) = args.video {
            self.video = Some(video);
            // If overlay option is set in config file & Video CLI argument is provided, then
            // automatically disable video overlay since it makes no sense with non live captured
            // frames.
            if self.overlay {
                eprintln!("warning: ignoring `overlay` option in configuration file while using `video` CLI argument.");
                self.overlay = false;
            }
        }
        if let Some(format) = args.format {
            self.format = format;
        }
        if args.overlay {
            // Overlay CLI flag is provided, but video is provided in configuration option: ignoring
            // overlay (date&time video overlay) option since it makes no sense with non live
            // captured frames.
            if self.video.is_some() {
                eprintln!("warning: ignoring `overlay` option while using `video` option in configuration file.");
            } else {
                // Overlay CLI flag is provided and live input is being used, so go ahead and
                // override configuration file with CLI flag provided.
                self.overlay = true;
            }
        }
        if args.quiet {
            self.quiet = true;
        }
        self
    }
}
