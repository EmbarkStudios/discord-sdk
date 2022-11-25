# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- markdownlint-disable blanks-around-headers no-duplicate-header blanks-around-lists -->

<!-- next-header -->
## [Unreleased] - ReleaseDate
## [0.3.1] - 2022-11-25
## [0.3.0] - 2022-03-02
### Changed
- [PR#20](https://github.com/EmbarkStudios/discord-sdk/pull/20) replaced `chrono` in favor of the (maintained) `time` crate.
- [PR#20](https://github.com/EmbarkStudios/discord-sdk/pull/20) updated `tracing-subscriber` and `parking_lot`.

## [0.2.1] - 2021-09-29
### Added
- [PR#19](https://github.com/EmbarkStudios/discord-sdk/pull/19) added an empty `register_app` implementation so that discord-sdk can be compiled for most targets, even if it doesn't actually function on them.

## [0.2.0] - 2021-09-29
### Changed
- [PR#18](https://github.com/EmbarkStudios/discord-sdk/pull/18/files#diff-63746a89ece2f6f7c95c84f99391f83a19ba24ca9825c5d993708ff60069a298) combined the `voice_mute` and `voice_deafen` RPCs into a single `update_voice_settings` RPC.

### Fixed
- [PR#18](https://github.com/EmbarkStudios/discord-sdk/pull/18/files#diff-9a3c0ce63dd7af5cdc3486b6e68ea8c098d855cfeccd72c6c66c69a069b31022) fixed the deserialization of activity timestamps in relationship update events.
- [PR#18](https://github.com/EmbarkStudios/discord-sdk/pull/18/files#diff-30f15d38fcb3d2d1714f1501c5520975acb8e72cf1ca62b7ca024fdb2a7267fb) fixed the `disconnect_lobby_voice` method to actually send the correct RPC.

## [0.1.4] - 2021-09-16
### Added
- [PR#17](https://github.com/EmbarkStudios/discord-sdk/pull/17) added [Voice](https://discord.com/developers/docs/game-sdk/discord-voice) support. Even though this functionality is going to be deprecated and removed by Discord, it was fairly easy to implement so there is little harm.

## [0.1.3] - 2021-08-23
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
[Unreleased]: https://github.com/EmbarkStudios/discord-sdk/compare/0.3.1...HEAD
[0.3.1]: https://github.com/EmbarkStudios/discord-sdk/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/EmbarkStudios/discord-sdk/compare/0.2.1...0.3.0
[0.2.1]: https://github.com/EmbarkStudios/discord-sdk/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/EmbarkStudios/discord-sdk/compare/0.1.4...0.2.0
[0.1.4]: https://github.com/EmbarkStudios/discord-sdk/compare/0.1.3...0.1.4
[0.1.3]: https://github.com/EmbarkStudios/discord-sdk/compare/0.1.2...0.1.3
[0.1.2]: https://github.com/EmbarkStudios/discord-sdk/compare/0.1.1...0.1.2
[0.1.1]: https://github.com/EmbarkStudios/discord-sdk/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/EmbarkStudios/discord-sdk/compare/0.0.1...0.1.0
[0.0.1]: https://github.com/EmbarkStudios/discord-sdk/releases/tag/0.0.1
