# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.2.0] - 2020-07-30
### Breaking
- The minimal required Rust version is now 1.45

### Added
- Add `serde` feature which implements `Serialize` and `Deserialize` for atomic
  types as appropriate (#2)

### Changed
- Some methods previously gated by the `nightly` feature are now always
  available (they were stabilized in Rust 1.45).

### Removed
- Remove `nightly` feature.This means `atomig` no longer uses
  `cfg(target_has_atomic)` gates. They will be added once they get stabilized.


## 0.1.0 - 2019-07-24
### Added
- Everything.


[Unreleased]: https://github.com/LukasKalbertodt/atomic/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/LukasKalbertodt/atomic/compare/v0.1.0...v0.2.0
