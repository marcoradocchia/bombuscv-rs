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
//!
//! # BombusCV
//!
//! Motion detection & video recording software based on **OpenCV**, built for research on
//! **Bumblebees** (hence the name).

pub mod args;
pub mod color;
pub mod config;
pub mod error;

use crate::error::ErrorKind;
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
        VideoCapture, VideoCaptureTrait, VideoCaptureTraitConst, VideoWriter, VideoWriterTrait,
        CAP_FFMPEG, CAP_PROP_FPS, CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH, CAP_V4L2,
    },
};
// use opencv::highgui;
use std::{os::raw::c_char, path::Path};

/// Video codecs.
pub enum Codec {
    MJPG,
    XVID,
    MP4V,
    H264,
}

impl Codec {
    /// Return the fourcc value associated to the video codec.
    fn fourcc(&self) -> i32 {
        // If no fourcc code can be obtained, video processing can't start, so it's fine to panic.
        match *self {
            Codec::MJPG => {
                VideoWriter::fourcc('M' as c_char, 'J' as c_char, 'P' as c_char, 'G' as c_char)
                    .expect("unable to generate MJPG fourcc code")
            }
            Codec::XVID => {
                VideoWriter::fourcc('X' as c_char, 'V' as c_char, 'I' as c_char, 'D' as c_char)
                    .expect("unable to generate XVID fourcc code")
            }
            Codec::MP4V => {
                VideoWriter::fourcc('m' as c_char, 'p' as c_char, '4' as c_char, 'v' as c_char)
                    .expect("unable to generate MP4V fourcc code")
            }
            Codec::H264 => {
                VideoWriter::fourcc('h' as c_char, '2' as c_char, '6' as c_char, '4' as c_char)
                    .expect("unable to generate H264 fourcc code")
            }
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
}

impl Grabber {
    /// Create an instance of the grabber from a camera input.
    ///
    /// # Parameters
    /// * index: _/dev/video<index>_ capture camera index
    /// * height: video capture desired frame height
    /// * width: video capture desired frame width
    /// * fps: video capture desired framerate
    /// * quiet: mute stdout output
    ///
    /// # Note
    ///
    /// Wherever the requested video capture parameters (height, width, fps) are not available for
    /// the given video capture device, OpenCV selects the closest available values.
    pub fn new(index: i32, height: i32, width: i32, fps: i32) -> Result<Self, ErrorKind> {
        // Generate Vector of VideoCapture parameters.
        let params = Vector::from_slice(&[
            CAP_PROP_FRAME_WIDTH,
            width,
            CAP_PROP_FRAME_HEIGHT,
            height,
            CAP_PROP_FPS,
            fps,
        ]);

        // Construct the VideoCapture object.
        match VideoCapture::new_with_params(index, CAP_V4L2, &params) {
            Ok(cap) => Ok(Self { cap }),
            Err(_) => Err(ErrorKind::InvalidCameraIndex),
        }
    }

    /// Create an instance of the grabber from a video file input.
    ///
    /// # Parameters
    /// * video: path of the video file
    /// * quiet: mute stdout output
    pub fn from_file(video: &Path) -> Result<Self, ErrorKind> {
        let video_path = video.to_str().expect("invalid UTF-8 video path");

        match VideoCapture::from_file(video_path, CAP_FFMPEG) {
            Ok(cap) => Ok(Self { cap }),
            Err(_) => Err(ErrorKind::InvalidVideoFile),
        }
    }

    pub fn get_height(&self) -> i32 {
        self.cap
            .get(CAP_PROP_FRAME_HEIGHT)
            .expect("unable to retrieve capture frame height") as i32
    }

    pub fn get_width(&self) -> i32 {
        self.cap
            .get(CAP_PROP_FRAME_WIDTH)
            .expect("unable to retrieve capture frame width") as i32
    }

    /// Return video capture frame Size.
    pub fn get_size(&self) -> Size {
        Size::new(self.get_width(), self.get_height())
    }

    /// Return video capture framerate.
    pub fn get_fps(&self) -> f64 {
        self.cap
            .get(CAP_PROP_FPS)
            .expect("unable to retrieve capture fps")
    }

    /// Grab video frame from camera and return it.
    pub fn grab(&mut self) -> Result<Frame, ErrorKind> {
        // Capture frame.
        let mut frame = Mat::default();
        if self.cap.read(&mut frame).is_ok() {
            Ok(Frame {
                frame,
                datetime: Local::now(),
            })
        } else {
            Err(ErrorKind::FrameDropped)
        }
    }
}

/// Implement Drop trait for the Grabber struct to release the VideoCapture on Grabber drop.
impl Drop for Grabber {
    fn drop(&mut self) {
        self.cap.release().expect("unable to release VideoCapture");
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
            // Initialize prev_frame as 640x480 empty frame: next grabbed frames will be
            // downscaled to this resolution and this initialization must be a valid Size for the
            // first frame comparison.
            prev_frame: unsafe { Mat::new_size(Size::new(640, 480), CV_8UC3).unwrap() },
        }
    }

    /// Receive grabbed frame and detect motion and returns:
    /// - `Ok`: if `Some(Frame)` motion detected; if `None` no motion detected.
    /// - `Err`: `frame` was empty and could not be processed.
    pub fn detect_motion(&mut self, frame: Frame) -> Result<Option<Frame>, ErrorKind> {
        // Create the resized_frame.
        let mut resized_frame = Mat::default();

        // Create two helper frames that will be used for image processing.
        let mut frame_one = Mat::default();
        let mut frame_two = Mat::default();

        // Contours must be C++ vector of vectors: std::vector<std::vector<cv::Point>>.
        let mut contours: Vector<Vector<Point>> = Vector::default();

        // If frame is empty return with Err.
        if frame.frame.empty() {
            return Err(ErrorKind::EmptyFrame);
        }

        // Downscale input frame (to 640x480) to reduce noise & computational weight.
        resize(
            &frame.frame,
            &mut resized_frame,
            // WARNING: check if chaning the aspect ratio causes any problem.
            Size::new(640, 480),
            0.,
            0.,
            INTER_LINEAR,
        )
        .expect("frame resizing failed");

        // Calculate absolute difference of pixel values.
        absdiff(&self.prev_frame, &resized_frame, &mut frame_one).expect("absdiff failed");

        // HELP: this are for graphical example
        // highgui::imshow("bombuscv", &frame_one).unwrap();
        // highgui::wait_key(1).unwrap();

        // Update the previous frame.
        self.prev_frame = resized_frame;

        // Convert from BGR colorspace to grayscale.
        cvt_color(
            &frame_one,
            &mut frame_two,
            COLOR_BGR2GRAY, // Color space conversion code (see #ColorConversionCodes).
            0, // Number of channels in the destination image; if the parameter is 0, the number of the channels is derived automatically from src and code.
        )
        .expect("cvt_color failed");

        // Apply gaussian blur
        gaussian_blur(
            &frame_two,
            &mut frame_one,
            Size::new(3, 3), // Kernel Size.
            21.,             // Gaussian kernel standard deviation in x direction.
            21.,             // Gaussian kernel standard deviation in y direction.
            BORDER_DEFAULT,
        )
        .expect("gaussian_blur failed");

        // Apply threshold.
        threshold(
            &frame_one,
            &mut frame_two,
            30.,           // Threshold value.
            255., // Maximum value to use with the #THRESH_BINARY and #THRESH_BINARY_INV thresholding types.
            THRESH_BINARY, // Thresholding type (see #ThresholdType).
        )
        .expect("threshold failed");

        // Dilate image.
        dilate(
            &frame_two,
            &mut frame_one,
            &Mat::default(), // Structuring element used for dilation; If elemenat=Mat(), a 3 x 3 rectangular structuring element is used.
            Point::new(-1, -1), // Position of the anchor within the element; default value (-1, -1) means that the anchor is at the element center.
            3,                  // Number of times dilation is applied.
            BORDER_CONSTANT,    // Pixel extrapolation method, see #BorderTypes.
            morphology_default_border_value().unwrap(), // Border value in case of a constant border.
        )
        .expect("dilate failed");

        // Find contours.
        find_contours(
            &frame_one,
            &mut contours, // Detected contours. Each contour is stored as a vector of points (e.g. std::vector<std::vectorcv::Point >).
            RETR_EXTERNAL, // Contour retrieval mode, see #RetrievalModes.
            CHAIN_APPROX_SIMPLE, // Contour approximation method, see #ContourApproximationModes.
            Point::new(0, 0), // Optional offset by which every contour point is shifted.
        )
        .expect("find_contours failed");

        // Count contours in the processed frame.
        Ok(match contours.is_empty() {
            // No motion was detected.
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
}

impl Writer {
    /// Create an instance of the writer.
    ///
    /// # Parameters
    /// * size: video frame
    /// * fps: video framerate
    /// * video_path: output video file path
    /// * overlay: date and time video overlay
    /// * quiet: mute stdout output
    pub fn new(
        video_path: &str,
        codec: Codec,
        fps: f64,
        size: Size,
        overlay: bool,
    ) -> Result<Self, ErrorKind> {
        // Construct the VideoWriter object.
        match VideoWriter::new(video_path, codec.fourcc(), fps, size, true) {
            Ok(writer) => Ok(Self { writer, overlay }),
            Err(_) => Err(ErrorKind::InvalidOutput),
        }
    }

    /// Write passed frame to the video file.
    pub fn write(&mut self, mut frame: Frame) -> Result<(), ErrorKind> {
        // Add date&time overlay.
        if self.overlay
            && put_text(
                &mut frame.frame,
                &frame.datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                Point::new(10, 40), // Bottom-left corner of the text string in the image.
                FONT_HERSHEY_DUPLEX, // Font type, see #hersheyfonts.
                1., // Font scale factor that is multiplied by the font-specific base size.
                Scalar::new(255., 255., 255., 1.), // Text color.
                2,  // Thickness.
                LineTypes::LINE_8 as i32, // Linetype.
                // true -> image data origin bottom-left corner
                // false -> top-left corner.
                false,
            )
            .is_err()
        {
            return Err(ErrorKind::TextOverlayFail);
        }

        // Write frame to video file.
        if self.writer.write(&frame.frame).is_err() {
            return Err(ErrorKind::FrameDropped);
        }

        Ok(())
    }
}

/// Implement Drop trait for the Writer struct to release the VideoWriter on Writer drop.
impl Drop for Writer {
    fn drop(&mut self) {
        self.writer
            .release()
            .expect("unable to release VideoWriter");
    }
}
