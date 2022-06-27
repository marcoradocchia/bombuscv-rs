<div align="center">
  <h1 align="center">BombusCV</h1>

  ![GitHub releases](https://img.shields.io/github/downloads/marcoradocchia/bombuscv-rs/total?color=%23a9b665&logo=github)
  ![GitHub source size](https://img.shields.io/github/languages/code-size/marcoradocchia/bombuscv-rs?color=ea6962&logo=github)
  ![GitHub open issues](https://img.shields.io/github/issues-raw/marcoradocchia/bombuscv-rs?color=%23d8a657&logo=github)
  ![GitHub open pull requests](https://img.shields.io/github/issues-pr-raw/marcoradocchia/bombuscv-rs?color=%2389b482&logo=github)
  ![GitHub sponsors](https://img.shields.io/github/sponsors/marcoradocchia?color=%23d3869b&logo=github)
  ![Crates.io downloads](https://img.shields.io/crates/d/bombuscv-rs?label=crates.io%20downloads&logo=rust)
  ![Crates.io version](https://img.shields.io/crates/v/bombuscv-rs?logo=rust&color=%23d8a657)
  ![Discord](https://img.shields.io/discord/985154521946816595?label=chat%20support&logo=discord&logoColor=%23ffff&color=%2389b482)
  ![GitHub license](https://img.shields.io/github/license/marcoradocchia/bombuscv-rs?color=%23e78a4e)
</div>

Motion detection & video recording software based on OpenCV, built for research
on **Bumblebees** (hence the name).

## Index

- [Use case](#use-case)
- [Examples](#examples)
- [Install](#install)
- [Usage](#usage)
- [Configuration](#configuration)
- [Changelog](#changelog)
- [ToDo](#todo)
- [Chat Support](#chat-support)
- [License](#license)

## Use case

This software was built to meet the need of tracking, and/or recording clips of
marked Bumblebee individuals in a scientific research project. It has been used
with a
[Raspberry Pi 4](https://www.raspberrypi.com/products/raspberry-pi-4-model-b/)[^1]
and a
[Raspberry Pi HQ Camera](https://www.raspberrypi.com/products/raspberry-pi-high-quality-camera/)[^2]
pointed at the entrance of a _Bombus terrestris_ nest, in order to record clips
of the entry/exit events, based on motion. This considerably reduced the
storage space required for the recordings and completely removed the need of
post processing work, since it was only recording clips in which individuals
appeared in the video frame.

`bombuscv-rs` offers realtime motion detection & video recording[^3] using
camera input and can be directly used on fieldwork. However, using the `video`
option, live camera input can be replaced with a pre-recorded video file: this
is useful to _remove dead moments_ from videos and reduce/remove the need of
manual video trimming.

[^1]: 4GB of RAM memory, powered by a 30000mAh battery power supply, which
  means this setup can be also reproduced in locations where no AC is available
[^2]: 12.3 megapixel _Sony IMX477_ sensor
[^3]: Based on hardware (RasberryPi 4 at the moment of writing can handle
  640x480 resolution at 60fps)

## Examples

Below a brief example of the produced video output:

<div align="center">

https://user-images.githubusercontent.com/74802223/171311278-c5caf303-832f-46f6-a4cc-a3e05f823349.mp4

</div>

More examples can be found on [YouTube](https://www.youtube.com/channel/UCkSjz-EAjhEvtUcY-ruYkrA).

## Install

For installation on *RaspberryPi* check [Install on RaspberryPi
4](#install-on-raspberrypi-4).

### Requirements

This program requires a working installation of **OpenCV** (`>=4.5.5`).
Building OpenCV from source is recommended (if you're going to build OpenCV
from source make sure to also install OpenCV dependencies), although it should
work with precompiled packages in your distro's repositories (it has been
tested with success on *ArchLinux* with the `extra/opencv` package).

### Using Cargo

A package is available at [crates.io](https://crates.io/crates/bombuscv-rs). In
order to install it run `cargo install bombuscv-rs` in your shell[^4].

[^4]: Assuming Rust installed

### Install on RaspberryPi 4

It is strongly recommended to use a RaspberryPi 4 with at least 4GB of RAM.
Also, before trying to install, please enable *Legacy Camera* support under
*Interface options*  running `raspi-config` and reboot. Since installation on a
RaspberryPi may be a little bit *tricky*, an installation script is
provided[^5]. It takes care of updating & preparing the system, compiling
*OpenCV* and installing *Rustup* and finally **BombusCV**. You can run the
[instllation script](bombuscv-raspi.sh) using `curl`:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/marcoradocchia/bombuscv-rs/master/bombuscv-raspi.sh | sh
```

[^5]: RaspberryPi OS 64 bits required in order to install using the script

## Usage

```
bombuscv-rs 0.3.0
Marco Radocchia <marco.radocchia@outlook.com>
OpenCV based motion detection/recording software built for research on bumblebees.

USAGE:
    bombuscv [OPTIONS]

OPTIONS:
    -d, --directory <DIRECTORY>    Output video directory
    -f, --framerate <FRAMERATE>    Video capture framerate
        --format <FORMAT>          Output video filename format (see
                                   <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
                                   for valid specifiers)
    -h, --help                     Print help information
    -H, --height <HEIGHT>          Video capture frame height
    -i, --index <INDEX>            /dev/video<INDEX> capture camera index
        --no-color                 Disable colored output
    -o, --overlay                  Date&Time video overlay
    -q, --quiet                    Mute standard output
    -v, --video <VIDEO>            Video file as input
    -V, --version                  Print version information
    -W, --width <WIDTH>            Video capture frame width
```

Specifying `width`, `height` & `framerate` will make `bombuscv` probe the
capture device for the closest combination of values it can provide and select
them. In other words: if you required valid options, they will be used,
otherwhise `bombuscv` will adapt those to the closest available combination[^6].

Note that `video` option, which runs `bombuscv` with a pre-recorded video
input, is incompatible with `framerate`, `width`, `height` and `overlay`. Also,
if these options are specified in the configuration file, they are going to be
ignored. This because the first two are auto-detected from the input file while
the last makes no sense if used with a non-live video feed; same rules apply to
CLI arguments.

[^6]: Same rules apply to configuration file

## Configuration

All CLI options (except `video` and `no-color`) can be set in a *optional* configuration file
stored at `$XDG_CONFIG_HOME/bombuscv/config.toml` by default or at any other
location in the filesystem specified by setting `BOMBUSCV_CONFIG` environment
variable. CLI options/arguments/flags override those defined in the
configuration file. Below listed an example configuration file:
```toml
# be quiet (mute stdout)
quiet = false
# output video directory
directory = "~/output_directory/"
# output video filename format (see
# https://docs.rs/chrono/latest/chrono/format/strftime/index.html for valid specifiers)
format = "%Y-%m-%dT%H:%M:%S"

# The following options are ignored if bombuscv is run with `--video` option
# /dev/video<index> camera input
index = 0
# video capture frame width
width = 640
# video capture frame height
height = 480
# video capture framerate
framerate = 30
# date&time video overlay
overlay = true
```

## Changelog

Complete [CHANGELOG](CHANGELOG.md).

## ToDo

- [x] Provide build & install instructions in [README](README.md), as well as
  the instructions to install OpenCV.
- [x] Make install script for automated installation on RaspberryPi.
- [x] Passing `video` or `directory` options in the configuration file using
  `~/<path>` results in an error: in the Deserialize expanding `~` to
  absolute path is required.
- [x] Using `video`, _date&time_ overlay generated on frame grabbed makes no
  sense: disable video overlay while using `video` option.
- [x] Add option to specify custom config path using env variables.
- [x] Add option to specify (in config file or via CLI argument) a custom
  output video filename formatter (must be [chrono DateTime
  syntax](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)).
- [x] Add thread signalling to interrupt grabber thread and gracefully
  terminate the execution.
- [x] Move logic from `main` to newly defined `run`.

## Chat Support

Join *BombusCV's Discord server* for installation or usage chat support:

<div align="center">

[![Join our Discord server!](https://invidget.switchblade.xyz/srNGQEs2QA?language=en)](http://discord.gg/srNGQEs2QA)

</div>

## License

[GPLv3](LICENSE)
