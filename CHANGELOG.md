# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.3] - 2021-12-30
### Changed
- This library is now `no_std` compatible. All paths to `std` items were replaced by `core`.
  (Thanks @eivindbergem https://github.com/LukasKalbertodt/atomig/pull/5)

## [0.3.2] - 2021-10-02
### Added
- `Atom` and `AtomInteger` impl for `Option<std::num::NonZero*>`

## [0.3.1] - 2021-06-18
### Added
- `Atom` impl for `std::ptr::NonNull<T>` and `Option<NonNull<T>>`
- `Atom` and `AtomLogic` impl for `std::num::Wrapping<T>`
- `Atom` for `std::num::NonZero*` types

## [0.3.0] - 2021-06-18
### Changed
- **Breaking**: the minimal supported Rust version (MSRV) is now 1.53
- **Breaking**: Remove deprecated method `compare_and_swap`. This method is
  deprecated in std and can be easily replaced by `compare_exchange`. See
  `std` docs for the migration.
- **Breaking**: all items of traits in the `impls` module are now considered
  implementation detail and not part of the public API.
- **Breaking**: the traits in the `impls` module were refactored and a lot
  changed about all of them. But due to the previous point, you are not
  supposed to care anymore :P
- Make `fetch_update` available to all `Atomic<T>`, not only `T: AtomicInteger`.

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


[Unreleased]: https://github.com/LukasKalbertodt/atomig/compare/v0.3.3...HEAD
[0.3.3]: https://github.com/LukasKalbertodt/atomig/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/LukasKalbertodt/atomig/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/LukasKalbertodt/atomig/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/LukasKalbertodt/atomig/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/LukasKalbertodt/atomig/compare/v0.1.0...v0.2.0
