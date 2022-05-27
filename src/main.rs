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
use bombuscv_rs::{Codec, Grabber, MotionDetector, Writer, Frame};
use chrono::Local;
use config::Config;

fn main() {
    // Parse CLI arguments
    let args = Args::parse();
    // Parse config and override options with CLI arguments where provided
    let config = Config::parse().override_with_args(args);

    dbg!(&config);

    // Format video file path as <config.directory/date&time>
    let filename = Local::now()
        .format(&format!(
            "{}/%Y-%m-%dT%H:%M:%S.mkv",
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
        Codec::MJPG,
        config.overlay,
        config.quiet,
    );

    // Instance of the motion detector.
    // let _detector = MotionDetector::new();

    let mut frames: Vec<Frame> = Vec::new();
    for _ in 0..100 {
        frames.push(grabber.grab());
    }

    for frame in frames {
        writer.write(frame);
    }

    // for i in 0..50 {
    //     let frame = grabber.grab();
    //     // writer.write(frame);
    //     println!("{}", i);
    // }
}
