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
use bombuscv_rs::ResConversion;
use directories::BaseDirs;
use opencv::core::Size;
use serde::Deserialize;
use std::{
    env,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    process,
    string::String,
};
use validator::{Validate, ValidationError};

const VALID_RESOLUTIONS: [&str; 8] = [
    "480p", "576p", "720p", "768p", "900p", "1080p", "1440p", "2160p",
];

/// Retrieve `resolution` & `framerate` information using ffprobe.
///
/// # Note
/// This function is intended to be used to parse Config fileds when using pre-recorded video file
/// as input.
fn video_metadata(video_path: &Path) -> (String, f64) {
    let streams = match ffprobe::ffprobe(video_path) {
        Ok(metadata) => metadata.streams,
        Err(e) => {
            eprintln!("error: unable to retrieve `{video_path:?}` metadata '{e}'");
            process::exit(1);
        }
    };

    // Iterated over probed streams in search of the first video stream.
    for stream in streams {
        if let Some(codec_type) = stream.codec_type {
            if codec_type == "video" {
                // Retrieve resolution information.
                let resolution = if stream.width.is_some() && stream.height.is_some() {
                    let width = stream.width.unwrap();
                    let height = stream.height.unwrap();
                    let res_string = format!("{height}p");

                    // If `video` resolution is not one of the valid resolutions exit with error:
                    // height is being checked in the from_str() function, so let's check if the
                    // corresponding width matches (the video is actually 16:9 aspect ratio).
                    if Size::from_str(&res_string).width as i64 != width {
                        eprintln!("error: `video` is not a supported resolution.");
                        process::exit(1);
                    }

                    res_string
                } else {
                    eprintln!(
                        "error: unable to retrieve `resolution` information from '{:?}'",
                        video_path
                    );
                    process::exit(1);
                };

                // Retrieve framerate information.
                let framerate = match stream.avg_frame_rate.split_once('/') {
                    Some(framerate) => framerate,
                    None => {
                        eprintln!(
                            "error: unable to retrieve `framerate` information from '{:?}'",
                            video_path
                        );
                        process::exit(1);
                    }
                };
                let framerate =
                    framerate.0.parse::<f64>().unwrap() / framerate.1.parse::<f64>().unwrap();

                return (resolution, framerate);
            }
        }
    }
    eprintln!("error: `video` does not contain any valid video stream.");
    process::exit(1);
}

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
    match VALID_RESOLUTIONS.contains(&resolution) {
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

    /// Video file as input.
    #[serde(skip_deserializing)]
    pub video: Option<PathBuf>,

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

    /// Output video filename format (see
    /// <https://docs.rs/chrono/latest/chrono/format/strftime/index.html> for valid specifiers).
    #[serde(default = "default_format")]
    pub format: String,

    /// Enable Date&Time video overlay.
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
            video: None,
            framerate: default_framerate(),
            resolution: default_resolution(),
            directory: default_directory(),
            format: default_format(),
            overlay: false,
            quiet: false,
        }
    }
}

impl Config {
    /// Parse configuration from config file.
    pub fn parse() -> Self {
        let config = if let Some(base_dirs) = BaseDirs::new() {
            // Fetch the environment variables for BOMBUSCV_CONFIG to hold a custom path to store
            // the configuration file.
            let config_file = fs::read_to_string(match env::var("BOMBUSCV_CONFIG") {
                // BOMBUSCV_CONFIG env variable set, so expand home and use it as the config.
                Ok(path) => expand_home(Path::new(&path)),
                // BOMBUSCV_CONFIG env variable unset or invalid: use default config file location.
                Err(_) => base_dirs
                    .config_dir() // XDG base spec directories: ~/.config/bomsucv/config.toml
                    .join(Path::new("bombuscv/config.toml")),
            })
            .unwrap_or_default(); // On Err variant, return empty string (==> default config).

            // Parse toml configuration file.
            let config: Config = match toml::from_str(&config_file) {
                Err(e) => {
                    eprintln!("error [config]: invalid config '{e}', using defaults");
                    Config::default()
                }
                Ok(config) => config,
            };

            // Values passed the parsing, now validate config values.
            if let Err(errors) = config.validate() {
                // Iterate over validation errors and display error messages to stderr.
                eprintln!("error [config]: invalid configuration options, using defaults");
                for (err, msg) in errors.field_errors() {
                    eprintln!("\t-> '{}': {}", err, msg.first().unwrap());
                }

                // Values didn't pass validation, return default configuration and warn the user.
                Config::default()
            } else {
                config
            }
        } else {
            eprintln!("warning [config]: no valid config path found on the system, using defaults");
            Config::default()
        };

        config
    }

    /// Override configuration with command line arguments.
    pub fn override_with_args(mut self, args: Args) -> Self {
        if let Some(directory) = args.directory {
            self.directory = directory;
        }

        if let Some(format) = args.format {
            self.format = format;
        }

        if args.quiet {
            self.quiet = true;
        }

        // Input is video: override resolution & framerate from config or args & disable overlay.
        if let Some(video) = args.video {
            (self.resolution, self.framerate) = video_metadata(&video);

            self.video = Some(video);

            if self.overlay {
                self.overlay = false;
                eprintln!(
                    "warning [config]: ignoring `overlay` while using `video` option."
                );
            }

            if !self.quiet {
                println!(
                    "info [config]: using `video` original `resolution` and `framerate`, \
                    ignoring eventually specified values."
                );
            }

            return self;
        }

        if let Some(index) = args.index {
            self.index = index;
        }

        if let Some(framerate) = args.framerate {
            if self.video.is_some() {
                eprintln!("warning [args]: ignoring `framerate` while using `video` option (auto-detected parameter).")
            } else {
                self.framerate = framerate;
            }
        }

        if let Some(resolution) = args.resolution {
            if self.video.is_some() {
                eprintln!("warning [args]: ignoring `resolution` while using `video` option (auto-detected parameter).")
            } else {
                self.resolution = resolution;
            }
        }

        if args.overlay {
            // Overlay CLI flag is provided, but video is provided in configuration option: ignoring
            // overlay (date&time video overlay) option since it makes no sense with non live
            // captured frames.
            if self.video.is_some() {
                eprintln!("warning [args]: ignoring `overlay` option while using `video` option in configuration file.");
            } else {
                // Overlay CLI flag is provided and live input is being used, so go ahead and
                // override configuration file with CLI flag provided.
                self.overlay = true;
            }
        }


        self
    }
}
