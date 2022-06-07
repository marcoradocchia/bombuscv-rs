<div align="center">
  <h1 align="center">BombusCV</h1>

  ![GitHub releases](https://img.shields.io/github/downloads/marcoradocchia/bombuscv-rs/total?color=%23a9b665&logo=github)
  ![GitHub repo size](https://img.shields.io/github/repo-size/marcoradocchia/bombuscv-rs?color=%23ea6962&logo=github)
  ![GitHub open issues](https://img.shields.io/github/issues-raw/marcoradocchia/bombuscv-rs?color=%23d8a657&logo=github)
  ![GitHub open pull requests](https://img.shields.io/github/issues-pr-raw/marcoradocchia/bombuscv-rs?color=%2389b482&logo=github)
  ![GitHub sponsors](https://img.shields.io/github/sponsors/marcoradocchia?color=%23d3869b&logo=github)
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
[^3]: Based on hardware

## Examples

Below a brief example of the produced video output:

https://user-images.githubusercontent.com/74802223/171311278-c5caf303-832f-46f6-a4cc-a3e05f823349.mp4

## Install

Clone the repository and build the project with `cargo`:
```sh
git clone https://github.com/marcoradocchia/bombuscv-rs.git
cd bombuscv-rs
cargo build --release
```

<!-- TODO: insert a tutorial on how to compile/install OpenCV in order to use
it with Rust, maybe an installation script -->

## Usage

```
bombuscv-rs 0.1.0
Marco Radocchia <marco.radocchia@outlook.com>
OpenCV based motion detection/recording software built for research on bumblebees.

USAGE:
    bombuscv [OPTIONS]

OPTIONS:
    -d, --directory <DIRECTORY>      Output video directory
    -f, --framerate <FRAMERATE>      Video framerate
        --format <FORMAT>            Output video filename format (see
                                     https://docs.rs/chrono/latest/chrono/format/strftime/index.html
                                     for valid specifiers)
    -h, --help                       Print help information
    -i, --index <INDEX>              /dev/video<INDEX> capture camera index
    -o, --overlay                    Enable Date/Time video overlay
    -q, --quiet                      Mute standard output
    -r, --resolution <RESOLUTION>    Video resolution (standard 16:9 formats) [possible values:
                                     480p, 576p, 720p, 768p, 900p, 1080p, 1440p, 2160p]
    -v, --video <VIDEO>              Input video file
    -V, --version                    Print version information
```

## Configuration

All options can be set in a optional configuration file stored at
`$XDG_CONFIG_HOME/bombuscv/config.toml` by default or at any location in the
filesystem specified by setting `BOMBUSCV_CONFIG` environment variable. CLI
arguments/flags override options defined in the configuration file. Below
listed an example configuration file:
```toml
# /dev/video<index> camera input
index = 0
# input/output framerate (ignored if used with `video`)
framerate = 10.0
# input/output resolution (ignored if used with `video`)
# possible values (16:9 formats): 480p, 576p, 720p, 768p, 900p, 1080p, 1440p, 2160p
resolution = "720p"
# date&time video overlay (ignored if used with `video`)
overlay = true
# be quiet (mute stdout)
quiet = false
# output video directory
directory = "~/output_directory/"
# output video filename format (see
# https://docs.rs/chrono/latest/chrono/format/strftime/index.html for valid specifiers)
format = "%Y-%m-%dT%H:%M:%S"
# input video file (replaces live camera input; conflicts with index, overlay)
# video = "~/input_video.mkv"
```

## Changelog

Complete [CHANGELOG](CHANGELOG.md).

## ToDo

- [ ] Provide build & install instructions in [README](README.md), as well as
  the instructions to install OpenCV.
- [ ] Make install script for automated installation on RaspberryPi.
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

## License

[GPLv3](LICENSE)
