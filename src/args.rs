pub use clap::Parser;
use std::path::PathBuf;

/// Validate framerate CLI argument.
pub fn validate_framerate(framerate: &str) -> Result<(), String> {
    let err_msg = String::from("the framerate must be a positive floating point number.");
    match framerate.parse::<f64>() {
        Ok(framerate) => {
            if framerate < 0. {
                return Err(err_msg);
            }
            Ok(())
        }
        Err(_) => Err(err_msg),
    }
}

/// Validate output video directory.
pub fn validate_directory(directory: &str) -> Result<(), String> {
    let err_msg = String::from("the given path is not a directory");
    if !PathBuf::from(directory).is_dir() {
        return Err(err_msg);
    }
    Ok(())
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
    /// /dev/video<index> capture camera index.
    #[clap(short, long)]
    pub index: Option<u8>,

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

    /// Enable Date/Time video overlay.
    #[clap(short, long)]
    pub overlay: bool,

    /// Mute standard output.
    #[clap(short, long)]
    pub quiet: bool,
}
