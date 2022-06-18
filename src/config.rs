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
use serde::{de, Deserialize, Deserializer};
use std::{
    env,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    process,
    string::String,
};

const VALID_RESOLUTIONS: [&str; 8] = [
    "480p", "576p", "720p", "768p", "900p", "1080p", "1440p", "2160p",
];

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

/// Custom deserializer for `directory` field: automatically expands ~ and creates PathBuf.
fn deserialize_directory<'de, D>(d: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path = expand_home(&PathBuf::deserialize(d)?);
    match path.is_dir() {
        true => Ok(path),
        false => Err(de::Error::custom(
            "specified `directory` option is not a valid path",
        )),
    }
}

/// Custom deserializer for `framerate` field: checks if field value is >1.0.
fn deserialize_framerate<'de, D>(d: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let framerate = f64::deserialize(d)?;
    if framerate > 1. {
        Ok(framerate)
    } else {
        Err(de::Error::invalid_value(
            de::Unexpected::Float(framerate),
            &"a value > 1.0",
        ))
    }
}

/// Custom deserializer for `resolution` field: checks for `resolution` in possible values.
fn deserialize_resolution<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let resolution = String::deserialize(d)?;

    match VALID_RESOLUTIONS.contains(&resolution.as_str()) {
        true => Ok(resolution),
        false => Err(de::Error::invalid_value(
            de::Unexpected::Str(&resolution),
            &format!("{:?}", VALID_RESOLUTIONS).as_str(),
        )),
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
#[derive(Deserialize, Debug)]
pub struct Config {
    /// /dev/video<index> capture camera index.
    #[serde(default = "default_index")]
    pub index: u8,

    /// Video file as input.
    #[serde(skip_deserializing)]
    pub video: Option<PathBuf>,

    /// Video framerate.
    #[serde(
        default = "default_framerate",
        deserialize_with = "deserialize_framerate"
    )]
    pub framerate: f64,

    /// Video resolution (standard 16:9 formats).
    #[serde(
        default = "default_resolution",
        deserialize_with = "deserialize_resolution"
    )]
    pub resolution: String,

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
        if let Some(base_dirs) = BaseDirs::new() {
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
            toml::from_str(&config_file).unwrap_or_else(|e| {
                // Configuration parsing failed: print error and use defaults.
                eprintln!("error [config]: {e}");
                println!("warning [config]: using defaults");
                Config::default()
            })
        } else {
            eprintln!("error [config]: no valid config path found on the system");
            println!("warning [config]: using defaults");
            Config::default()
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

        if args.quiet {
            self.quiet = true;
        }

        // Input is video: override resolution & framerate from config or args & disable overlay.
        if let Some(video) = args.video {
            self.video = Some(video);
            if self.overlay {
                self.overlay = false;
                eprintln!("warning [config]: ignoring `overlay` while using `video` option.");
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
            self.framerate = framerate;
        }

        if let Some(resolution) = args.resolution {
            self.resolution = resolution;
        }

        if args.overlay {
            self.overlay = true;
        }

        self
    }
}
