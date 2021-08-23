# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- markdownlint-disable blanks-around-headers no-duplicate-header blanks-around-lists -->

<!-- next-header -->
## [Unreleased] - ReleaseDate
### Changed
- [PR#16](https://github.com/EmbarkStudios/discord-sdk/pull/16) exposed the `Snowflake` type publicly, as there are cases where you might need to use it directly as it is the underlying type for most of the unique identifiers throught the SDK.

### Fixed
- [PR#16](https://github.com/EmbarkStudios/discord-sdk/pull/16) fixed regions to use `kebab-case` instead of `snake_case`, and add the `st-pete` region, which is apparently a voice region that can be used, but isn't listed in `/voice/regions`.

## [0.1.2] - 2021-08-11
### Fixed
- [PR#14](https://github.com/EmbarkStudios/discord-sdk/pull/14) fixed an issue where the `RELATIONSHIP_UPDATE` event actually uses stringized timestamps in the activity information, rather than the normal `i64` timestamps in eg `SET_ACTIVITY`.
- [PR#14](https://github.com/EmbarkStudios/discord-sdk/pull/14) fixed an issue with timestamps being converted into `chrono::DateTime<Utc>` with the wrong unit, resulting in date times far in the future.
- [PR#14](https://github.com/EmbarkStudios/discord-sdk/pull/14) added more sanitization to `crate::activity::ActivityBuilder` to prevent strings with just whitespace being sent to Discord as that results in API failures.

## [0.1.1] - 2021-07-28
### Added
- [PR#10](https://github.com/EmbarkStudios/discord-sdk/pull/10) added `ActivityBuilder::start_timestamp` and `ActivityBuilder::end_timestamp` as well as implementing `IntoTimestamp` for `i64`. Thanks [@Ewpratten](https://github.com/Ewpratten)!

## [0.1.0] - 2021-07-21
### Added
- Initial version with basic support for [Activities](https://discord.com/developers/docs/game-sdk/activities), [Lobbies](https://discord.com/developers/docs/game-sdk/lobbies), [Overlay](https://discord.com/developers/docs/game-sdk/overlay), [Relationships](https://discord.com/developers/docs/game-sdk/relationships), [Users](https://discord.com/developers/docs/game-sdk/users), and application registration.

## [0.0.1] - 2021-06-04
### Added
- Initial crate squat

<!-- next-url -->
[Unreleased]: https://github.com/EmbarkStudios/discord-sdk/compare/0.1.2...HEAD
[0.1.2]: https://github.com/EmbarkStudios/discord-sdk/compare/0.1.1...0.1.2
[0.1.1]: https://github.com/EmbarkStudios/discord-sdk/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/EmbarkStudios/discord-sdk/compare/0.0.1...0.1.0
[0.0.1]: https://github.com/EmbarkStudios/discord-sdk/releases/tag/0.0.1
