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

use std::{
    fmt::{self, Display, Formatter},
    io::{self, Write},
};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

/// Type of the message represented.
#[derive(Debug)]
pub enum MsgType {
    Info,
    Warn,
    Error,
    Hint,
}

/// Message Colorizer.
///
/// # Fields
///
/// * color_when: variant of the ColorChoice enum from `termcolor` library
/// * msg_type: variant of the `MsgType` representing wheter the message content is **Info**
///     (*green* prefix), **Warn** (*yellow* prefix) or **Error** (*red* prefix)
/// * prefix: portion of the message preceding `:` (i.e. "**error:** this is an error message")
/// * body: body of the message (i.e. "error: **this is an error message**")
#[derive(Debug)]
pub struct Colorizer {
    color_when: ColorChoice,
    msg_type: MsgType,
    prefix: String,
    body: String,
}

impl Colorizer {
    /// Construct Colorizer instance with empty prefix & body.
    pub fn empty(msg_type: MsgType, no_color: bool) -> Self {
        Self {
            color_when: color_choice(&msg_type, no_color),
            msg_type,
            prefix: "".to_string(),
            body: "".to_string(),
        }
    }

    /// Construct a Colorizer instance.
    pub fn new(
        msg_type: MsgType,
        no_color: bool,
        prefix: impl ToString,
        body: impl ToString,
    ) -> Self {
        Self {
            color_when: color_choice(&msg_type, no_color),
            msg_type,
            prefix: prefix.to_string(),
            body: body.to_string(),
        }
    }

    /// Print colored output.
    pub fn print(&self) -> io::Result<()> {
        let mut color = ColorSpec::new();

        let writer = match self.msg_type {
            MsgType::Info => {
                color.set_fg(Some(Color::Green)).set_bold(true);
                BufferWriter::stdout(self.color_when)
            }
            MsgType::Warn => {
                color.set_fg(Some(Color::Yellow)).set_bold(true);
                BufferWriter::stderr(self.color_when)
            }
            MsgType::Error => {
                color.set_fg(Some(Color::Red)).set_bold(true);
                BufferWriter::stderr(self.color_when)
            }
            MsgType::Hint => {
                color.set_fg(Some(Color::Cyan)).set_dimmed(true);
                BufferWriter::stdout(self.color_when)
            }
        };

        let mut buffer = writer.buffer();
        buffer.set_color(&color)?;
        write!(&mut buffer, "{}:", self.prefix)?;
        buffer.reset()?;
        writeln!(&mut buffer, " {}", self.body)?;

        writer.print(&buffer)
    }

    /// Update prefix & body field values.
    pub fn update(&mut self, prefix: impl ToString, body: impl ToString) {
        self.prefix = prefix.to_string();
        self.body = body.to_string();
    }
}

/// Print with no colors.
impl Display for Colorizer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.prefix, self.body)?;
        Ok(())
    }
}

/// Returns `ColorChoice::Auto` or `ColorChoice::Never` based on Stdout/Stderr being the terminal
/// or not or no_color being true/false.
fn color_choice(msg_type: &MsgType, no_color: bool) -> ColorChoice {
    if no_color {
        return ColorChoice::Never;
    }

    let stream = match msg_type {
        MsgType::Info | MsgType::Hint => atty::Stream::Stdout,
        MsgType::Warn | MsgType::Error => atty::Stream::Stderr,
    };

    if atty::is(stream) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    }
}
