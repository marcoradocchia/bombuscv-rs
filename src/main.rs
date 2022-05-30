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

#[cfg(test)]
mod test;

mod args;
mod config;

use args::{Args, Parser};
use bombuscv_rs::{Codec, Grabber, MotionDetector, Writer};
use chrono::Local;
use config::Config;
use std::sync::mpsc;
use std::thread;

fn main() {
    // Parse CLI arguments.
    let args = Args::parse();
    // Parse config and override options with CLI arguments where provided.
    let config = Config::parse().override_with_args(args);

    // Print config options.
    println!("{:#?}", &config);

    // panic!();

    // Format video file path as <config.directory/date&time>.
    let filename = Local::now()
        .format(&format!(
            "{}/%Y-%m-%dT%H:%M:%S.mkv",
            config.directory.to_str().unwrap()
        ))
        .to_string();

    // Instance of the frame grabber.
    let mut grabber = match config.video {
        Some(video) => Grabber::from_file(&video, config.quiet),
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

    // Create channels for message passing between threads.
    let (raw_tx, raw_rx) = mpsc::channel();
    let (proc_tx, proc_rx) = mpsc::channel();

    // Spawn frame grabber thread:
    // this thread captures frames and passes them to the motion detecting thread.
    let grabber_handle = thread::spawn(move || {
        loop {
            if raw_tx.send(grabber.grab()).is_err() {
                grabber.release();
                break;
            }
        }
    });

    // Spawn motion detecting thread:
    // this thread receives frames from the grabber thread, processes it and if motion is detected,
    // passes the frame to the frame writing thread.
    thread::spawn(move || {
        for frame in raw_rx {
            if let Some(frame) = detector.detect_motion(frame) {
                if proc_tx.send(frame).is_err() {
                    eprintln!("error: frame dropped");
                };
            }
        }
    });

    // Spawn frame writer thread:
    // this thread receives the processed frames by the motion detecting thread and writes them in
    // the output video file.
    thread::spawn(move || {
        for frame in proc_rx {
            writer.write(frame);
        }
        writer.release();
    });

    grabber_handle.join().unwrap();
}
