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

use bombuscv_rs::{
    args::{Args, Parser},
    color::{Colorizer, MsgType},
    config::Config,
    Codec, Grabber, MotionDetector, Writer,
};
use chrono::Local;
use signal_hook::{consts::SIGINT, flag::register};
use std::io;
use std::{
    path::Path,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

fn main() -> io::Result<()> {
    // Parse CLI arguments.
    let args = Args::parse();
    // Parse config file and override options with CLI arguments.
    let config = match Config::parse() {
        Ok(config) => config,
        Err(e) => {
            Colorizer::new(MsgType::Error, args.no_color, "error [config]", e).print()?;
            Colorizer::new(
                MsgType::Warn,
                args.no_color,
                "warning",
                "using default configuration",
            )
            .print()?;
            Config::default()
        }
    }
    .override_with_args(args);

    // Format video file path as <config.directory/date&time>.
    let filename = Local::now()
        .format(
            config
                .directory
                // Output video file name (derived by file format) + extension.
                .join(Path::new(&config.format).with_extension("mkv"))
                // Convert Path object to string.
                .to_str()
                .unwrap(),
        )
        .to_string();

    // Instance of the frame grabber.
    let grabber = match &config.video {
        // VideoCapture is video file.
        Some(video) => Grabber::from_file(video),
        // VideoCapture is live camera.
        None => Grabber::new(
            config.index.into(),
            config.height.into(),
            config.width.into(),
            config.framerate.into(),
        ),
    };
    let grabber = match grabber {
        Ok(grabber) => grabber,
        Err(e) => {
            Colorizer::new(MsgType::Error, config.no_color, "error", e).print()?;
            process::exit(1);
        }
    };

    // Print info.
    if !config.quiet {
        let mut colorizer = Colorizer::empty(MsgType::Info, config.no_color);

        let input = if let Some(video) = &config.video {
            video.display().to_string()
        } else {
            format!("/dev/video{}", &config.index)
        };

        let messages = vec![
            ("==> Input", input),
            ("==> Framerate", grabber.get_fps().to_string()),
            ("==> Printing overlay", format!("{}", config.overlay)),
            ("==> Output video file", filename.clone()),
            ("==> Frame size", format!("{}x{}", grabber.get_width(), grabber.get_height())),
        ];

        for msg in messages {
            colorizer.update(msg.0, msg.1);
            colorizer.print()?;
        }
    }

    // Instance of the motion detector.
    let detector = MotionDetector::new();

    // Instance of the frame writer.
    let writer = match Writer::new(
        &filename,
        Codec::XVID,
        grabber.get_fps(),
        grabber.get_size(),
        config.overlay,
    ) {
        Ok(writer) => writer,
        Err(e) => {
            Colorizer::new(MsgType::Error, config.no_color, "error", e).print()?;
            process::exit(1);
        }
    };

    // Save memory dropping `filename`.
    drop(filename);

    // Run the program.
    run(grabber, detector, writer, config.no_color)?;

    // Gracefully terminated execution.
    if !config.quiet {
        Colorizer::new(MsgType::Info, config.no_color, "\nbombuscv", "done!").print()?;
    }

    Ok(())
}

/// Run `bombuscv`: spawn & join frame grabber, detector and writer threads.
fn run(
    mut grabber: Grabber,
    mut detector: MotionDetector,
    mut writer: Writer,
    no_color: bool,
) -> io::Result<()> {
    // Create channels for message passing between threads.
    // NOTE: using mpsc::sync_channel (blocking) to avoid channel size
    // growing indefinitely, resulting in infinite memory usage.
    let (raw_tx, raw_rx) = mpsc::sync_channel(100);
    let (proc_tx, proc_rx) = mpsc::sync_channel(100);

    // Spawn frame grabber thread:
    // this thread captures frames and passes them to the motion detecting thread.
    let grabber_handle = thread::spawn(move || -> io::Result<()> {
        let term = Arc::new(AtomicBool::new(false));
        // Register signal hook for SIGINT events: in this case error is unrecoverable, so report
        // it to the user & exit process with code error code.
        if let Err(e) = register(SIGINT, Arc::clone(&term)) {
            Colorizer::new(
                MsgType::Error,
                no_color,
                "fatal error",
                format!("unable to register SIGINT hook '{e}'"),
            )
            .print()?;
            process::exit(1);
        };

        // Start grabber loop: loop guard is 'received SIGINT'.
        while !term.load(Ordering::Relaxed) {
            let frame = match grabber.grab() {
                Ok(frame) => frame,
                Err(e) => {
                    Colorizer::new(MsgType::Warn, no_color, "warning", e).print()?;
                    continue;
                }
            };

            // Grab frame and send it to the motion detection thread.
            if raw_tx.send(frame).is_err() {
                break;
            }
        }

        Ok(())
    });

    // Spawn motion detection thread:
    // this thread receives frames from the grabber thread, processes it and if motion is detected,
    // passes the frame to the frame writing thread.
    let detector_handle = thread::spawn(move || -> io::Result<()> {
        // Loop over received frames from the frame grabber.
        for frame in raw_rx {
            match detector.detect_motion(frame) {
                // Valid frame is received.
                Ok(val) => {
                    // Motion has been detected: send frame to the video writer.
                    if let Some(frame) = val {
                        if proc_tx.send(frame).is_err() {
                            Colorizer::new(
                                MsgType::Warn,
                                no_color,
                                "warning",
                                "unable to send processed frame to video output",
                            )
                            .print()?;
                        };
                    }
                }
                // Last captured frame was an empty frame: no more input is provided, interrupt the
                // thread (break the loop).
                Err(_) => break,
            }
        }

        Ok(())
    });

    // Spawn frame writer thread:
    // this thread receives the processed frames by the motion detecting thread and writes them in
    // the output video output.
    let writer_handle = thread::spawn(move || -> io::Result<()> {
        // Loop over received frames from the motion detector.
        for frame in proc_rx {
            // Write processed frames (motion detected) to the video output.
            if let Err(e) = writer.write(frame) {
                Colorizer::new(MsgType::Warn, no_color, "warning", e).print()?;
            };
        }

        Ok(())
    });

    // Join all threads.
    grabber_handle.join().expect("cannot join grabber thread")?;
    detector_handle
        .join()
        .expect("cannot join detector thread")?;
    writer_handle.join().expect("cannot join writer thread")?;

    Ok(())
}
