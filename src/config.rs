use crate::args::Args;
use directories::BaseDirs;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process,
};
use validator::{Validate, ValidationError};

/// Expands `~` in `path` to absolute HOME path.
pub fn expand_home(path: &Path) -> PathBuf {
    let home = match BaseDirs::new() {
        Some(base_dirs) => base_dirs.home_dir().to_path_buf(),
        None => {
            eprintln!("error: unable to find home directory");
            process::exit(1);
        }
    };

    match path.strip_prefix("~") {
        Ok(path) => home.join(path),
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
            overlay: false,
            quiet: false,
        }
    }
}

impl Config {
    /// Parse configuration from config file.
    pub fn parse() -> Self {
        if let Some(base_dirs) = BaseDirs::new() {
            // Lin: /home/alice/.config/
            // Win: C:\Users\Alice\AppData\Roaming\
            // Mac: /Users/Alice/Library/Application Support/
            let config_dir = base_dirs.config_dir();

            let config_file =
                fs::read_to_string(config_dir.join(Path::new("bombuscv/config.toml")))
                    .unwrap_or_default();

            let config: Config = match toml::from_str(&config_file) {
                Err(e) => {
                    eprintln!("error: broken config '{e}', using defaults");
                    Config::default()
                }
                Ok(config) => config
            };

            // if values passed the parsing, validate config values
            match config.validate() {
                // if values pass validation, return parsed configuration
                Ok(_) => config,
                // if values don't pass validation, return default configuration
                Err(errors) => {
                    // TODO: not very elegant
                    // gather all the invalid value into a sting and display it as an error message
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
        }
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
        }
        if args.video.is_some() {
            self.video = args.video;
        }
        if args.overlay {
            self.overlay = true;
        }
        if args.quiet {
            self.quiet = true;
        }
        self
    }
}
