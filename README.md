<p align="center">
# BombusCV
</p>

![GitHub releases](https://img.shields.io/github/downloads/marcoradocchia/bombuscv-rs/total?color=%23ea6962&logo=github&style=flat-square)
![GitHub repo size](https://img.shields.io/github/repo-size/marcoradocchia/bombuscv-rs?style=flat-square)
![GitHub license](https://img.shields.io/github/license/marcoradocchia/bombuscv-rs?style=flat-square)
![GitHub open issues](https://img.shields.io/github/issues-raw/marcoradocchia/bombuscv-rs?style=flat-square)
![GitHub open pull requests](https://img.shields.io/github/issues-pr-raw/marcoradocchia/bombuscv-rs?style=flat-square)
![Crates.io downloads](https://img.shields.io/crates/d/bombuscv-rs?style=flat-square)
![Crates.io version](https://img.shields.io/crates/v/bombuscv-rs?style=flat-square)

Motion detection & video recording software based on OpenCV, built for research
on **Bumblebees** (hence the name).

## Index

- [Use case](#use-case)
- [Examples](#examples)
- [Build & Install](#build-&-install)
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
pointed at the entrance of a _Bombus Terrestris_ nest, in order to record clips
of the entry/exit events, based on motion. This considerably reduced the
storage space required for the recordings and completely removed the need of
post processing work, since it was only recording clips in which individuals
apppeared in the video frame.

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
<!-- TODO -->

## Build & Install

Clone the repository and build the project with `cargo`:
```sh
git clone https://github.com/marcoradocchia/bombuscv-rs.git
cd bombuscv-rs
cargo build --release
```

<!-- TODO: insert a tutorial on how to compile/install OpenCV in order to use
it with Rust -->

## Usage

```
bombuscv-rs 0.1.0
Marco Radocchia <marco.radocchia@outlook.com>
OpenCV based motion detection/recording software built for research on bumblebees.

USAGE:
    bombuscv-rs [OPTIONS]

OPTIONS:
    -d, --directory <DIRECTORY>      Output video directory
    -f, --framerate <FRAMERATE>      Video framerate
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
`$XDG_CONFIG_HOME/bombuscv/config.toml`. CLI arguments/flags override options
defined in the configuration file.
Below listed an example configuration file:
```toml
# /dev/video<index> camera input
index = 0
# input/output framerate (output only in case of `video` option used)
framerate = 10.0
# input/output resolution (output only in case of `video` option used)
resolution = "720p"
# date&time video overlay
overlay = true
# be quiet (mute stdout)
quiet = false
# output video directory
directory = "~/output_directory/"
# input video file (replaces live camera input; conflicts with index, overlay)
# video = "~/input_video.mkv"
```

## Changelog

Complete [CHANGELOG](CHANGELOG.md).

## ToDo

- [x] Passing `video` or `directory` options in the configuration file using
  `~/<path>` results in an error: in the Deserialize expanding `~` to
  absolute path is required
- [x] Using `video`, _date&time_ overlay generated on frame grabbed makes no
  sense: disable video overlay while using `video` option

## License

[GPLv3](LICENSE)
