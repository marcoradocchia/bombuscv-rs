use std::fs;

use super::*;

#[test]
/// Time the frame acquisition/processing/writing and check if framerate is >= than the required
/// framerate passed in the configuration or CLI arguments.
fn check_framerate() {

    // Parse CLI arguments
    let args = Args::parse();
    // Parse config and override options with CLI arguments where provided
    let config = Config::parse().override_with_args(args);

    // Print configuration options.
    println!("==> {config:#?}");

    // Format video file path as <config.directory/date&time>
    let filename = Local::now()
        .format(&format!(
            "{}/%Y-%m-%dT%H:%M:%S.mp4",
            config.directory.to_str().unwrap()
        ))
        .to_string();

    // Instance of the frame grabber.
    let mut grabber = Grabber::new(
        config.index as i32,
        &config.resolution,
        config.framerate,
        config.quiet,
    );

    // Instance of the frame writer.
    let mut writer = Writer::new(
        &config.resolution,
        config.framerate,
        &filename,
        Codec::MP4V,
        config.overlay,
        config.quiet,
    );

    // Set the number of frames to record based on the required framerate in order to obtain 10
    // seconds recording test.
    let n_frames = config.framerate as usize * 10;

    // Time the frame acquisition/writing.
    let start = Local::now();
    for _ in 0..n_frames {
        let frame = grabber.grab();
        writer.write(frame);
    }
    let time = Local::now();
    let time = (time - start).num_microseconds().unwrap() as f64 / 1e6;
    println!("==> Test duration: {time}");

    let fps = n_frames as f64 / time;
    println!("==> Framerate: {:?}", fps);

    // Remove generated video file.
    if fs::remove_file(filename).is_ok() {
        println!("==> Test video file removed.")
    }

    // If tested framerate is >= than given framerate test is passed;
    assert!(fps >= config.framerate);
}
