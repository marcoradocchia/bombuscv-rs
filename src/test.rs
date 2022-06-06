use crate::{Codec, Config, Grabber, Local, MotionDetector, Path, Writer};
use bombuscv_rs::Frame;
use directories::BaseDirs;
use std::{time::Instant, fs};

#[test]
fn sync_frame_processing_avg_time() {
    // Number of frames to acquire.
    const N: usize = 500;

    // Generate the absolute path for HOME.
    let home = BaseDirs::new().unwrap().home_dir().to_path_buf();

    // Parse CLI arguments.
    let config = Config {
        index: 0,
        framerate: 60.,
        resolution: String::from("720p"),
        video: Some(home.join("test_720.mkv")),
        directory: home,
        format: String::from("output"),
        overlay: false,
        quiet: false,
    };

    // Format video file path as <config.directory/date&time>.
    let filename = Local::now()
        .format(
            config
                // Output video file directory.
                .directory
                // Output video file name (derived by file format) + extension.
                .join(Path::new(&config.format).with_extension("mkv"))
                // Convert Path object to string;
                .to_str()
                .unwrap(),
        )
        .to_string();

    // Print config options if config.quiet is false.
    if !config.quiet {
        println!("{:#?}", &config);
    }

    // Instance of the frame grabber.
    let mut grabber = match &config.video {
        // VideoCapture is video file.
        Some(video) => Grabber::from_file(video, config.quiet),
        // VideoCapture is live camera.
        None => Grabber::new(
            config.index.into(),
            &config.resolution,
            config.framerate,
            config.quiet,
        ),
    };

    // Instance of the motion detector.
    let mut detector = MotionDetector::new();

    // Instance of the frame writer.
    let mut writer = Writer::new(
        &config.resolution,
        config.framerate,
        &filename,
        Codec::XVID,
        config.overlay,
        config.quiet,
    );

    // Vector of frames to test performance on.
    let mut frames: Vec<Frame> = Vec::with_capacity(N);
    let mut detected_frames = 0;

    // Acquire N frames.
    for _ in 0..N {
        frames.push(grabber.grab());
    }

    // Save the start time.
    let start = Instant::now();
    for frame in frames {
        match detector.detect_motion(frame) {
            Ok(frame) => {
                if let Some(frame) = frame {
                    // If frame is detected, write it to the file.
                    writer.write(frame);
                    // Count the detected frames.
                    detected_frames += 1;
                }
            }
            Err(_) => panic!("not enaugh frames to run the test!")
        }
    }

    //Calculate the elapsed time to process motion detection on all the N frmaes.
    let tot_dur_ns = start.elapsed();
    let dur_ns = tot_dur_ns.div_f32(N as f32);
    println!("==> # saved frames: {}", detected_frames);
    println!("==> processing motion detection took: {:?}", tot_dur_ns);
    println!(
        "==> processing motion detection per frame took (avg): {:?}",
        dur_ns
    );
    let max = 1e3 / config.framerate as f32;
    println!("==> max value allowed: {}ms", max);

    // Remove output file.
    fs::remove_file(filename).expect("unable to remove output file.");

    assert!(dur_ns.subsec_micros() <= (max * 1e3) as u32);
}
