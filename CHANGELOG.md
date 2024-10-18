# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.2] - 2024-10-18
- Add `Send + Sync + Unpin` bound to `PrimitiveAtom::Impl`.
  As a consequence, these traits are now implemented for `Atomic<T>` unconditionally.
  ([#14](https://github.com/LukasKalbertodt/atomig/issues/14))

## [0.4.1] - 2024-03-04
- Add const fn `Atomic::from_impl` to allow creating `static` `Atomic`s [#11](https://github.com/LukasKalbertodt/atomig/pull/11)
- Add `Atom` & `AtomLogic` impl for small, pow2-sized integer arrays [#12](https://github.com/LukasKalbertodt/atomig/pull/12)
- Fix CI badge in README

## [0.4.0] - 2022-04-09
### Changed
- **Breaking**: the minimal supported Rust version (MSRV) is now 1.60

- Use `cfg(target_has_atomic)` to conditionally disable some impls. Previously,
  the whole crate just failed to compile on targets like `thumbv7em-none-eabi`,
  as `AtomicU64` is not available there. Now it compiles, but there is no
  `Atom` impl for `u64` or `f64`, for example. This is not yet an optimal
  solution, as platform support for atomics is not just differentiated by size,
  but also by whether they support some features. Sadly, for those, there are
  no `cfg` flags available on stable yet.

- Relax `derive` for structs by only requiring `Atom` for the inner field, not
  `PrimitiveAtom`.

- Derive macro uses `syn`, `quote` and `proc-macro2` in version 1.x now. This is
  semantically irrelevant for you, but might remove the pre-1.x versions of
  these crates from your dependency graph if atomig was the last to use them.


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


[Unreleased]: https://github.com/LukasKalbertodt/atomig/compare/v0.4.2...HEAD
[0.4.2]: https://github.com/LukasKalbertodt/atomig/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/LukasKalbertodt/atomig/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/LukasKalbertodt/atomig/compare/v0.3.3...v0.4.0
[0.3.3]: https://github.com/LukasKalbertodt/atomig/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/LukasKalbertodt/atomig/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/LukasKalbertodt/atomig/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/LukasKalbertodt/atomig/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/LukasKalbertodt/atomig/compare/v0.1.0...v0.2.0
