# Changelog

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
