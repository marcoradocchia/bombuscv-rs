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
//
//! # BombusCV
//! OpenCV based motion detection/recording software built for research on bumblebees.

use chrono::{DateTime, Local};
use opencv::{
    core::{absdiff, Point, Scalar, Size, Vector, BORDER_CONSTANT, BORDER_DEFAULT, CV_8UC3},
    imgproc::{
        cvt_color, dilate, find_contours, gaussian_blur, morphology_default_border_value, put_text,
        resize, threshold, LineTypes, CHAIN_APPROX_SIMPLE, COLOR_BGR2GRAY, FONT_HERSHEY_DUPLEX,
        INTER_LINEAR, RETR_EXTERNAL, THRESH_BINARY,
    },
    prelude::{Mat, MatTraitConst},
    videoio::{
        VideoCapture, VideoCaptureTrait, VideoWriter, VideoWriterTrait, CAP_FFMPEG, CAP_PROP_FPS,
        CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH, CAP_V4L2,
    },
};
use std::{error::Error, fmt::Display, path::Path, process};

/// OpenCV related errors.
#[derive(Debug)]
pub enum FrameError {
    EmptyFrame,
}

/// Implement Error trait for FrameError enum.
impl Error for FrameError {
    fn description(&self) -> &str {
        match *self {
            FrameError::EmptyFrame => "empty frame",
        }
    }
}

/// Implement Display trait for FrameError enum.
impl Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            FrameError::EmptyFrame => write!(f, "error: empty frame"),
        }
    }
}

/// Trait implementations for resolution conversions.
pub trait ResConversion {
    fn from_str(res: &str) -> Self;
}

impl ResConversion for Size {
    /// Convert from string to opencv::core::Size using the standard 16:9 formats.
    fn from_str(res: &str) -> Self {
        match res {
            "480p" => Size::new(854, 480),
            "576p" => Size::new(1024, 576),
            "720p" => Size::new(1280, 720),
            "768p" => Size::new(1366, 768),
            "900p" => Size::new(1600, 900),
            "1080p" => Size::new(1920, 1080),
            "1440p" => Size::new(2560, 1440),
            "2160p" => Size::new(3840, 2160),
            res => {
                eprintln!("error: {res} is not a valid resolution");
                process::exit(1);
            }
        }
    }
}

/// Video codecs.
pub enum Codec {
    MJPG,
    XVID,
    MP4V,
}

impl Codec {
    /// Return the fourcc value associated to the video codec.
    fn fourcc(&self) -> i32 {
        match *self {
            Codec::MJPG => VideoWriter::fourcc('M' as i8, 'J' as i8, 'P' as i8, 'G' as i8).unwrap(),
            Codec::XVID => VideoWriter::fourcc('X' as i8, 'V' as i8, 'I' as i8, 'D' as i8).unwrap(),
            Codec::MP4V => VideoWriter::fourcc('m' as i8, 'p' as i8, '4' as i8, 'v' as i8).unwrap(),
        }
    }
}

/// Captured Frame.
///
/// # Fields
/// * frame: the video frame itself
/// * datetime: DateTime object representing the instant
pub struct Frame {
    pub frame: Mat,
    pub datetime: DateTime<Local>,
}

/// Video frame grabber.
///
/// # Fields
/// * cap: OpenCV VideoCapture instance
/// * quiet: mute standard output
pub struct Grabber {
    cap: VideoCapture,
    quiet: bool,
}

impl Grabber {
    /// Create an instance of the grabber from a camera input.
    ///
    /// # Parameters
    /// * index: _/dev/video<index>_ capture camera index
    /// * res: video resolution
    /// * fps: video framerate
    /// * quiet: mute stdout output
    pub fn new(index: i32, res: &str, fps: f64, quiet: bool) -> Self {
        // Generate Size object for resolution.
        let res = Size::from_str(res);
        // Generate Vector of VideoCapture parameters.
        let params = Vector::from_slice(&[
            CAP_PROP_FRAME_WIDTH,
            res.width,
            CAP_PROP_FRAME_HEIGHT,
            res.height,
            CAP_PROP_FPS,
            fps as i32,
        ]);

        // Construct the VideoCapture object.
        let cap = match VideoCapture::new_with_params(index, CAP_V4L2, &params) {
            Ok(cap) => cap,
            Err(e) => {
                eprintln!("unable to open camera '{e}'");
                process::exit(1);
            }
        };

        Self { cap, quiet }
    }

    /// Create an instance of the grabber from a video file input.
    ///
    /// # Parameters
    /// * video: path of the video file
    /// * quiet: mute stdout output
    pub fn from_file(video: &Path, quiet: bool) -> Self {
        let video_path = video.to_str().unwrap();

        let cap = match VideoCapture::from_file(video_path, CAP_FFMPEG) {
            Ok(cap) => cap,
            Err(e) => {
                eprintln!("unable to open video file `{video_path}` '{e}'");
                process::exit(1);
            }
        };

        Self { cap, quiet }
    }

    /// Grab video frame from camera and return it.
    pub fn grab(&mut self) -> Frame {
        // Capture frame.
        let mut frame = Mat::default();
        if self.cap.read(&mut frame).is_err() && !self.quiet {
            println!("warning: cap frame dropped")
        }

        Frame {
            frame,
            datetime: Local::now(),
        }
    }
}

/// Implement Drop trait for the Grabber struct to release the VideoCapture on Grabber drop.
impl Drop for Grabber {
    fn drop(&mut self) {
        if self.cap.release().is_err() {
            println!("error: unable to release VideoCapture");
        };
    }
}

/// Motion detector.
///
/// # Fields
/// * prev_frame: previous frame to make comparisons
pub struct MotionDetector {
    prev_frame: Mat,
}

impl Default for MotionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl MotionDetector {
    /// Create an instance of the MotionDetector.
    pub fn new() -> Self {
        Self {
            // Initialize prev_frame as 480p resolution empty frame: next grabbed frames will be
            // downscaled to this resolution and this initialization must be a valid Size for the
            // first frame comparison.
            prev_frame: unsafe { Mat::new_size(Size::from_str("480p"), CV_8UC3).unwrap() },
        }
    }

    /// Receive grabbed frame and detect motion, returning `Some(Frame)` if motion is detected,
    /// `None` if no motion is detected.
    pub fn detect_motion(&mut self, frame: Frame) -> Result<Option<Frame>, FrameError> {
        // Create the resized_frame.
        let mut resized_frame = Mat::default();
        // Create two helper frames that will be used for image processing.
        let mut frame_one = Mat::default();
        let mut frame_two = Mat::default();
        // Contours must be C++ vector of vectors: std::vector<std::vector<cv::Point>>.
        let mut contours: Vector<Vector<Point>> = Vector::default();

        // If frame is empty return with error.
        if frame.frame.empty() {
            return Err(FrameError::EmptyFrame);
        }

        // Downscale input frame (to 480p resolution) to reduce noise & computational weight.
        resize(
            &frame.frame,
            &mut resized_frame,
            Size::from_str("480p"),
            0.,
            0.,
            INTER_LINEAR,
        )
        .unwrap();

        // Calculate absolute difference of pixel values.
        absdiff(&self.prev_frame, &resized_frame, &mut frame_one).expect("error: absdiff failed");
        // Update the previous frame.
        self.prev_frame = resized_frame;

        // Convert from BGR colorspace to grayscale.
        cvt_color(
            &frame_one,
            &mut frame_two,
            COLOR_BGR2GRAY, // Color space conversion code (see #ColorConversionCodes).
            0, // Number of channels in the destination image; if the parameter is 0, the number of the channels is derived automatically from src and code.
        )
        .expect("error: cvt_color failed");

        // Apply gaussian blur
        gaussian_blur(
            &frame_two,
            &mut frame_one,
            Size::new(3, 3), // Kernel Size.
            21.,             // Gaussian kernel standard deviation in x direction.
            21.,             // Gaussian kernel standard deviation in y direction.
            BORDER_DEFAULT,
        )
        .expect("error: gaussian_blur failed");

        // Apply threshold.
        threshold(
            &frame_two,
            &mut frame_one,
            30.,           // Threshold value.
            255., // Maximum value to use with the #THRESH_BINARY and #THRESH_BINARY_INV thresholding types.
            THRESH_BINARY, // Thresholding type (see #ThresholdType).
        )
        .expect("error: threshold failed");

        // Dilate image.
        dilate(
            &frame_one,
            &mut frame_two,
            &Mat::default(), // Structuring element used for dilation; If elemenat=Mat(), a 3 x 3 rectangular structuring element is used.
            Point::new(-1, -1), // Position of the anchor within the element; default value (-1, -1) means that the anchor is at the element center.
            3,                  // Number of times dilation is applied.
            BORDER_CONSTANT,    // Pixel extrapolation method, see #BorderTypes.
            morphology_default_border_value().unwrap(), // Border value in case of a constant border.
        )
        .expect("error: dilate failed");

        // Find contours.
        find_contours(
            &frame_two,
            &mut contours, // Detected contours. Each contour is stored as a vector of points (e.g. std::vector<std::vectorcv::Point >).
            RETR_EXTERNAL, // Contour retrieval mode, see #RetrievalModes.
            CHAIN_APPROX_SIMPLE, // Contour approximation method, see #ContourApproximationModes.
            Point::new(0, 0), // Optional offset by which every contour point is shifted.
        )
        .expect("error: find_contours failed");

        // TEST: uncomment the following line to test the changes in the detector parameters.
        // println!("{}", contours.len());

        // Count contours in the processed frame.
        Ok(match contours.is_empty() {
            // No motion was found.
            true => None,
            // Motion was found, return original video frame.
            false => Some(frame),
        })
    }
}

/// Video frame writer.
///
/// # Fields
/// * writer: OpenCV
/// * overlay: date&time video overlay
/// * quiet: mute standard output
pub struct Writer {
    writer: VideoWriter,
    overlay: bool,
    quiet: bool,
}

impl Writer {
    /// Create an instance of the writer.
    ///
    /// # Parameters
    /// * res: video resolution
    /// * fps: video framerate
    /// * video_path: output video file path
    /// * overlay: date and time video overlay
    /// * quiet: mute stdout output
    pub fn new(
        res: &str,
        fps: f64,
        video_path: &str,
        codec: Codec,
        overlay: bool,
        quiet: bool,
    ) -> Self {
        // Generate Size object for resolution.
        let res = Size::from_str(res);
        // Construct the VideoWriter object.
        let writer = match VideoWriter::new(video_path, codec.fourcc(), fps, res, true) {
            Ok(writer) => writer,
            Err(e) => {
                eprintln!("unable to create video writer {e}");
                process::exit(1);
            }
        };

        Self {
            writer,
            overlay,
            quiet,
        }
    }

    /// Write passed frame to the video file.
    pub fn write(&mut self, mut frame: Frame) {
        // Add date&time overlay.
        if self.overlay
            && put_text(
                &mut frame.frame,
                &frame.datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                Point { x: 10, y: 40 }, // Bottom-left corner of the text string in the image.
                FONT_HERSHEY_DUPLEX,    // Font type, see #hersheyfonts.
                1., // Font scale factor that is multiplied by the font-specific base size.
                Scalar::new(255., 255., 255., 1.), // Text color.
                2,  // Thickness.
                LineTypes::FILLED as i32, // Linetype.
                // true -> image data origin bottom-left corner
                // false -> top-left corner.
                false,
            )
            .is_err()
            && !self.quiet
        {
            println!("warning: unable to print text overlay")
        };

        // Write frame to video file.
        if self.writer.write(&frame.frame).is_err() && !self.quiet {
            println!("warning: frame dropped");
        }
    }
}

/// Implement Drop trait for the Writer struct to release the VideoWriter on Writer drop.
impl Drop for Writer {
    fn drop(&mut self)
    {
        if self.writer.release().is_err() {
            println!("error: unable to release VideoWriter");
        };
    }
}
