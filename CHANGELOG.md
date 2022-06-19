# Changelog

## [Unreleased] <!-- 0.3.0 -->

Updated
[README](https://github.com/marcoradocchia/bombuscv-rs/blob/master/README.md)
*Use case*, *Install*, *Usage* and *Configuration* sections.

### Changed

- Moved `resolution` option to `width` & `height` options in both configuration
  file and CLI arguments: now custom resolution (and aspect ratio) and
  framerate can be specified and `bobmuscv` will adapt those to the closest
  combination of resolution and framerate the capture device provides.

### Removed

- Dependency `ffprobe`: pre-recorded video *resolution* and *framerate*,
  required to construct the `VideoWriter`, are now obtained using **OpenCV**'s
  `VideoCapture` getters methods;

## [0.2.0] - 2022-06-14

Updated
[README](https://github.com/marcoradocchia/bombuscv-rs/blob/master/README.md)
*Examples*, *Usage* and *Configuration* sections.

### Changed

- Updated `bombuscv-raspi.sh` script to install **OpenCV** `v4.6.0`.

### Fixed

- Issue marcoradocchia/bombuscv-rs#1 which prevented `bombuscv` to autodetect
  video framerate and resolution on `video` CLI option used.

### Removed

- Option to specify `video` in the configuration file in favor of passing video
  file via CLI argument.
- Dependency `validator`: moved config file options validation to `serde`.

## [0.1.1] - 2022-06-11

Updated
[README](https://github.com/marcoradocchia/bombuscv-rs/blob/master/README.md)
*Install* section.

### Added

- [bombuscv-raspi.sh](https://github.com/marcoradocchia/bombuscv-rs/blob/master/bombuscv-raspi.sh)
  for automated build & installation on RaspberryPi (RaspberryPi OS AArch64).

### Fixed

- Issue marcoradocchia/bombuscv-rs#2 which caused compilation errors on AArch64 systems.

## [0.1.0] - 2022-06-06

- Initial release.
