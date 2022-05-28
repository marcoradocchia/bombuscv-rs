//! # BombusCV
//! OpenCV based motion detection/recording software built for research on bumblebees.

use chrono::{DateTime, Local};
use opencv::{
    core::{absdiff, Point, Scalar, Size, Vector, BORDER_CONSTANT, BORDER_DEFAULT},
    imgproc::{
        cvt_color, dilate, find_contours, gaussian_blur, morphology_default_border_value, put_text,
        threshold, LineTypes, CHAIN_APPROX_SIMPLE, COLOR_BGR2GRAY, FONT_HERSHEY_DUPLEX,
        RETR_EXTERNAL, THRESH_BINARY,
    },
    prelude::{Mat, MatTraitConst},
    videoio::{
        VideoCapture, VideoCaptureTrait, VideoWriter, VideoWriterTrait, CAP_PROP_FPS,
        CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH, CAP_V4L2,
    },
};

/// Trait implementations for resolution conversions.
pub trait ResConversion {
    fn from_str(res: &str) -> Self;
}

impl ResConversion for Size {
    /// Convert from string to opencv::core::Size using the standard 16:9 formats.
    fn from_str(res: &str) -> Self {
        match res {
            "480p" => Size {
                width: 854,
                height: 480,
            },
            "576p" => Size {
                width: 1024,
                height: 576,
            },
            "720p" => Size {
                width: 1280,
                height: 720,
            },
            "768p" => Size {
                width: 1366,
                height: 768,
            },
            "900p" => Size {
                width: 1600,
                height: 900,
            },
            "1080p" => Size {
                width: 1920,
                height: 1080,
            },
            "1440p" => Size {
                width: 2560,
                height: 1440,
            },
            "2160p" => Size {
                width: 3840,
                height: 2160,
            },
            res => panic!("{} is not a valid resolution", res),
        }
    }
}

/// List of video codecs.
pub enum Codec {
    MJPG,
    XVID,
    YUYV,
    H264,
}

impl Codec {
    /// Returns the fourcc associated to the video codec.
    fn fourcc(&self) -> i32 {
        match *self {
            Codec::MJPG => VideoWriter::fourcc('M' as i8, 'J' as i8, 'P' as i8, 'G' as i8).unwrap(),
            Codec::XVID => VideoWriter::fourcc('X' as i8, 'V' as i8, 'I' as i8, 'D' as i8).unwrap(),
            Codec::YUYV => VideoWriter::fourcc('Y' as i8, 'U' as i8, 'Y' as i8, 'V' as i8).unwrap(),
            Codec::H264 => VideoWriter::fourcc('H' as i8, '2' as i8, '6' as i8, '4' as i8).unwrap(),
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
    /// Create an instance of the grabber.
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
            Err(e) => panic!("unable to open camera {:?}", e),
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

/// Motion detector.
///
/// # Fields
/// * prev_frame: previous frame to make comparisons
pub struct MotionDetector {
    prev_frame: Mat,
}

impl MotionDetector {
    pub fn new() -> Self {
        Self {
            prev_frame: Mat::default(),
        }
    }

    /// Dequeue frames and detect motion.
    pub fn detect_motion(&mut self, frame: Frame) -> Option<Frame> {
        // Try to grab a frame from the frame queue and process motion detection on success.
        // If motion is detected, append current dequeued frame to the proc_frames queue in order
        // to be processed.

        // TODO: consider adding resize to 480p (does it perform better?)
        let mut frame_one = Mat::default();
        let mut frame_two = Mat::default();

        // Calculate absolute difference of pixel values.
        absdiff(&self.prev_frame, &frame.frame, &mut frame_one).unwrap();

        // Convert from BGR colorspace to grayscale.
        cvt_color(
            &frame_one,
            &mut frame_two,
            COLOR_BGR2GRAY, // Color space conversion code (see #ColorConversionCodes).
            0, // Number of channels in the destination image; if the parameter is 0, the number of the channels is derived automatically from src and code.
        )
        .unwrap();

        // Apply gaussian blur
        gaussian_blur(
            &frame_two,
            &mut frame_one,
            Size {
                // Kernel Size.
                width: 3,
                height: 3,
            },
            21., // Gaussian kernel standard deviation in x direction.
            21., // Gaussian kernel standard deviation in y direction.
            BORDER_DEFAULT,
        )
        .unwrap();

        // Apply threshold.
        threshold(
            &frame_two,
            &mut frame_one,
            30.,           // Threshold value.
            255., // Maximum value to use with the #THRESH_BINARY and #THRESH_BINARY_INV thresholding types.
            THRESH_BINARY, // Thresholding type (see #ThresholdType).
        )
        .unwrap();

        // Dilate image.
        dilate(
            &frame_one,
            &mut frame_two,
            // TODO: check structuring element.
            &Mat::default(), // Structuring element used for dilation; If elemenat=Mat(), a 3 x 3 rectangular structuring element is used.
            Point { x: -1, y: -1 }, // Position of the anchor within the element; default value (-1, -1) means that the anchor is at the element center.
            3,                      // Number of times dilation is applied.
            BORDER_CONSTANT,        // Pixel extrapolation method, see #BorderTypes.
            morphology_default_border_value().unwrap(), // Border value in case of a constant border.
        )
        .unwrap();

        // Find contours.
        find_contours(
            &frame_two,
            // TODO: check
            &mut frame_one, // Detected contours. Each contour is stored as a vector of points (e.g. std::vector<std::vectorcv::Point >).
            RETR_EXTERNAL,  // Contour retrieval mode, see #RetrievalModes.
            CHAIN_APPROX_SIMPLE, // Contour approximation method, see #ContourApproximationModes.
            Point { x: 0, y: 0 }, // Optional offset by which every contour point is shifted.
        )
        .unwrap();

        // Now frame_one contains contours, ready to be counted.
        // If are found => motion => return Option<Frame> to be written in video file.
        match frame_one.total() {
            0 => None,
            _ => Some(frame),
        }
    }
}

/// Implement Default trait for MotionDetector.
impl Default for MotionDetector {
    fn default() -> Self {
        Self::new()
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
            Err(e) => panic!("unable to create video writer {:?}", e),
        };

        Self {
            writer,
            overlay,
            quiet,
        }
    }

    /// Write frame to the video file.
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
