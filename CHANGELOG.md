# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2021-03-14
### Changed
- Rename `Format::{GA8, GA16}` to `Format::{Ga8, Ga16}` to be consistent with other `Format` enum values
- Rename `raw::IfPresent` to `raw::ChunkAvailable`
- Create type aliases for non-wrapped chunks and move all to `raw::chunk`
### Added
- Added `Reader<T>::raw_context()` to enable access to `&RawContext`.

## [0.1.0-alpha.6] - 2021-03-14
### Changed
- Update libspng to 264476a
- Now zlib is linked to statically
- Added `zlib-ng` crate feature to opt-in to `zlib-ng`, a fork of zlib with better performance

### Fixed
- Now the `DEP_SPNG_INCLUDE` environment variable is correctly set to the include directory that contains libspng headers

## [0.1.0-alpha.5] - 2020-06-14
### Changed
- Update libspng to 71a71a6
### Added
- Grayscale output formats

## [0.1.0-alpha.4] - 2020-05-29
### Added
- Expose the `RawContext` API
- Add `spng::decode` for simple use cases
### Changed
- Update libspng to f47ed26
### Added
- Detect CPU target features and enable corresponding options in libspng

## [0.1.0-alpha.3] - 2020-03-13

## [0.1.0-alpha+2] - 2020-03-13
### Fixed
- Buffered stream decoding now reads from the source buffer until the
  destination buffer is full.

## [0.1.0-alpha+1] - 2020-03-13
### Added
- Initial rust wrapper with minimal API surface
- Initial native bindings to [libspng] `master` ([2079ef6])

[Unreleased]: https://github.com/aloucks/spng-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/aloucks/spng-rs/releases/tag/v0.1.0
[0.1.0-alpha.6]: https://github.com/aloucks/spng-rs/releases/tag/v0.1.0-alpha.6
[0.1.0-alpha.5]: https://github.com/aloucks/spng-rs/releases/tag/v0.1.0-alpha.5
[0.1.0-alpha.4]: https://github.com/aloucks/spng-rs/releases/tag/v0.1.0-alpha.4
[0.1.0-alpha.3]: https://github.com/aloucks/spng-rs/releases/tag/v0.1.0-alpha.3
[0.1.0-alpha+2]: https://github.com/aloucks/spng-rs/releases/tag/v0.1.0-alpha+2
[0.1.0-alpha+1]: https://github.com/aloucks/spng-rs/releases/tag/v0.1.0-alpha+1

[libspng]: https://libspng.org
[2079ef6]: https://github.com/randy408/libspng/tree/2079ef6f223feea2570b537c047c9140a5b72551