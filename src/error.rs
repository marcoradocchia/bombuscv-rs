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

use std::fmt::{self, Display, Formatter};

/// BombusCV error kinds.
#[derive(Debug)]
pub enum ErrorKind {
    /// Occurs when config file path cannot be determined.
    ConfigNotFound,
    /// Occurs when parsing a broken configuration file.
    BrokenConfig(String),
    /// Occurs when VideoCapture is unable to open camera.
    InvalidCameraIndex,
    /// Occurs when VideoCapture is unable to open video file.
    InvalidVideoFile,
    /// Occurs when VideoWriter is unable to open video output file.
    InvalidOutput,
    /// Occurs when VideoCapture read fails.
    FrameDropped,
    /// Occurs when VideoCapture returns an empty frame.
    EmptyFrame,
    /// Occurs when VideoWriter fails to print text overlay on video frame.
    TextOverlayFail
}

impl Display for ErrorKind {
    /// Description of the error case, where relevant.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigNotFound => Some("no valid config path found".to_string()),
            Self::BrokenConfig(msg) => Some(msg.to_string()),
            Self::InvalidCameraIndex => Some("unable to open camera by index".to_string()),
            Self::InvalidVideoFile => Some("unable to open video file".to_string()),
            Self::InvalidOutput => Some("unable to open video output file".to_string()),
            Self::FrameDropped => None,
            Self::EmptyFrame => Some("empty video frame".to_string()),
            Self::TextOverlayFail => Some("unable to print text overlay".to_string()),
        }.unwrap_or_default().fmt(f)
    }
}
