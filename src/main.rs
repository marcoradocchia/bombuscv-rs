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

mod args;
mod config;

use args::{Args, Parser};
use bombuscv_rs::{Codec, Grabber, MotionDetector, Writer};
use chrono::Local;
use config::Config;
use signal_hook::{consts::SIGINT, flag::register};
use std::{
    path::Path,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

fn main() {
    // Parse CLI arguments.
    let args = Args::parse();

    // Parse config and override options with CLI arguments where provided.
    let config = Config::parse().override_with_args(args);

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

    // Print config options if config.quiet is not true.
    if !config.quiet {
        if let Some(video) = &config.video {
            println!("==> Input video file: {}", video.display())
        } else {
            println!(
                "==> Resolution: {}\n==> Framerate: {}",
                &config.resolution, &config.framerate
            )
        }
        println!(
            "==> Output video file: {}\n==> Printing overlay: {}",
            filename, &config.overlay
        );
    }

    // Instance of the frame grabber.
    let mut grabber = match config.video {
        // VideoCapture is video file.
        Some(video) => Grabber::from_file(&video, config.quiet),
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

    // Save memory dropping filename.
    drop(filename);

    // Create channels for message passing between threads.
    let (raw_tx, raw_rx) = mpsc::channel();
    let (proc_tx, proc_rx) = mpsc::channel();

    // Spawn frame grabber thread:
    // this thread captures frames and passes them to the motion detecting thread.
    let grabber_handle = thread::spawn(move || {
        let term = Arc::new(AtomicBool::new(false));
        // Register signal hook for SIGINT events: catch eventual error, report it to the user &
        // exit process with code error code.
        if let Err(e) = register(SIGINT, Arc::clone(&term)) {
            eprintln!("unable to register signal hook '{e}'");
            process::exit(1);
        };

        // Start grabber loop: loop guard is `received SIGINT`.
        while !term.load(Ordering::Relaxed) {
            // Grab frame and send it to the motion detection thread.
            if raw_tx.send(grabber.grab()).is_err() {
                break;
            }
        }
    });

    // Spawn motion detection thread:
    // this thread receives frames from the grabber thread, processes it and if motion is detected,
    // passes the frame to the frame writing thread.
    let detector_handle = thread::spawn(move || {
        // Loop over received frames from the frame grabber.
        for frame in raw_rx {
            match detector.detect_motion(frame) {
                // Valid frame is received.
                Ok(val) => {
                    // Motion has been detected: send frame to the video writer.
                    if let Some(frame) = val {
                        if proc_tx.send(frame).is_err() {
                            eprintln!("error: frame dropped");
                        };
                    }
                }
                // Last captured frame was an empty frame: no more input is provided, interrupt the
                // thread (break the loop).
                Err(_) => break,
            }
        }
    });

    // Spawn frame writer thread:
    // this thread receives the processed frames by the motion detecting thread and writes them in
    // the output video file.
    let writer_handle = thread::spawn(move || {
        // Loop over received frames from the motion detector.
        for frame in proc_rx {
            // Write processed frames (motion detected) to the video file.
            writer.write(frame);
        }
    });

    // Join all threads.
    grabber_handle.join().unwrap();
    detector_handle.join().unwrap();
    writer_handle.join().unwrap();

    // Gracefully terminated execution.
    if !config.quiet {
        println!("Done.");
    }
}
