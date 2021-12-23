# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.2] - 2021-12-23
### Changed
 - Eliminated undefined behaviour on pin toggle

## [0.5.1] - 2021-12-23
### Fixed
 - Fix panic on pin toggle

## [0.5.0] - 2021-12-23
### Changed
 - Rename `wire` methods `as_…` to `connect_…`

### Added
 - Extended tests to increase test coverage for `wire`

## [0.4.2] - 2021-12-23
### Added
 - Reimplemented `ToggleableOutputPin` for `wire` with correct locking

## [0.4.1] - 2021-12-23
### Changed
 - Extended tests to increase test coverage

## [0.4.0] - 2021-12-23
### Changed
 - Updated `embedded-hal` to `1.0.0-alpha.6`

### Removed
 - `ToggleableOutputPin` implementation for `wire` was removed, as the default
   implementation was removed from `embedded-hal` and an explicit implementation
   on the wire requires more work

## [0.3.1] - 2021-02-05
### Fixed
 - Fixed implementation of `embedded_hal::digital::ToggleableOutputPin` for wire pins

## [0.3.0] - 2021-02-05
### Added
 - Add implementation of `embedded_hal::digital::ToggleableOutputPin`

## [0.2.0] - 2021-02-01
### Added
 - Add implementation of `embedded_hal::digital::StatefulOutputPin`
 
### Fixed
 - Minor spelling correction in documentation.

## [0.1.0] - 2021-01-12
### Added
 - First public release.
