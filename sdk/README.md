<!-- markdownlint-disable no-inline-html first-line-h1 no-duplicate-heading -->

<div align="center">

# `⚔️ discord-sdk`

**An (unofficial) open source Rust implementation of the [Discord Game SDK](https://discord.com/developers/docs/game-sdk/sdk-starter-guide).**

[![Embark](https://img.shields.io/badge/embark-open%20source-blueviolet.svg)](https://embark.dev)
[![Embark](https://img.shields.io/badge/discord-ark-%237289da.svg?logo=discord)](https://discord.gg/dAuKfZS)
[![Crates.io](https://img.shields.io/crates/v/discord-sdk.svg)](https://crates.io/crates/discord-sdk)
[![Docs](https://docs.rs/discord-sdk/badge.svg)](https://docs.rs/discord-sdk)
[![dependency status](https://deps.rs/repo/github/EmbarkStudios/discord-sdk/status.svg)](https://deps.rs/repo/github/EmbarkStudios/discord-sdk)
[![Build status](https://github.com/EmbarkStudios/discord-sdk/workflows/CI/badge.svg)](https://github.com/EmbarkStudios/discord-sdk/actions)

</div>

## Why not use this?

- This project is not official and is using a largely undocumented protocol that Discord could change/break at any time in the future.
- There is already a [Rust wrapper](https://crates.io/crates/discord_game_sdk) for the official Game SDK.
- Your project is not also in Rust. We may add a C API for this crate in the future, but for now this is a Rust only project.

## Why use this?

- You use Rust for your project and want to integrate features such as [rich presence/activities](https://discord.com/rich-presence) provided by Discord.
- You don't want to have a dependency on a closed source, shared library.
- You like to live dangerously (though this library does also have some automated tests!).

## Implemented Features

### [Activities (Rich Presence)](https://discord.com/developers/docs/developer-tools/game-sdk#activities)

#### Commands

- [x] [Update Activity](https://discord.com/developers/docs/game-sdk/activities#updateactivity)
- [x] [Clear Activity](https://discord.com/developers/docs/game-sdk/activities#clearactivity)
- [x] [Send Join Request Reply](https://discord.com/developers/docs/game-sdk/activities#sendrequestreply)
- [x] [Send Invite](https://discord.com/developers/docs/game-sdk/activities#sendinvite)
- [x] [Accept Invite](https://discord.com/developers/docs/game-sdk/activities#acceptinvite)

#### Events

- [x] [Join](https://discord.com/developers/docs/game-sdk/activities#onactivityjoin)
- [x] [Spectate](https://discord.com/developers/docs/game-sdk/activities#onactivityspectate)
- [x] [Join Request](https://discord.com/developers/docs/game-sdk/activities#onactivityjoinrequest)
- [x] [Invite](https://discord.com/developers/docs/game-sdk/activities#onactivityinvite)

#### Other

- [x] [Application Registration (Windows, Linux, Mac)](https://discord.com/developers/docs/game-sdk/activities#registercommand)

#### Other

### [Overlay](https://discord.com/developers/docs/developer-tools/game-sdk#overlay)

**NOTE**: These are only tested insofar as the protocol is (probably) correct, however, the overlay is currently extremely limited, and so we were unable to test that the overlay commands _actually_ function correctly since our primary project is Vulkan.

> [Note, there are a few other cases that overlay will not work with. The overlay is currently not supported for Mac, games with Vulkan support, and generally old games.](https://support.discord.com/hc/en-us/articles/217659737-Games-Overlay-101)

Also note, the SDK itself and its documentation uses the utterly confusing terminology of Un/Locked when talking about the overlay, this crate instead uses `Visibility`, as in `Visible` or `Hidden`.

#### Commands

- [x] [Toggle Visibility](https://discord.com/developers/docs/game-sdk/overlay#setlocked)
- [x] [Open Activity Invite](https://discord.com/developers/docs/game-sdk/overlay#openactivityinvite)
- [x] [Open Guild Invite](https://discord.com/developers/docs/game-sdk/overlay#openguildinvite)
- [x] [Open Voice Settings](https://discord.com/developers/docs/game-sdk/overlay#openvoicesettings) - **NOTE**: AFAICT, if your application does not have the overlay enabled (eg, because it is Vulkan or a CLI or whatnot), this will **crash Discord**.

#### Events

- [x] [Overlay Update](https://discord.com/developers/docs/game-sdk/overlay#ontoggle)

### [Relationships](https://discord.com/developers/docs/game-sdk/relationships)

#### Commands

- [x] [Get Relationships](https://discord.com/developers/docs/game-sdk/relationships#first-notes) - **NOTE**: This command is not really exposed directly from the regular Game SDK, but is implicitly executed by the SDK during intialization.

#### Events

- [x] [Relationship Update](https://discord.com/developers/docs/game-sdk/relationships#onrelationshipupdate)

### [Users](https://discord.com/developers/docs/developer-tools/game-sdk#users)

#### Commands

- [x] [Get Current User](https://discord.com/developers/docs/game-sdk/users#getcurrentuser)
- [ ] [Get User](https://discord.com/developers/docs/game-sdk/users#getuser)

#### Events

- [x] [Current User Update](https://discord.com/developers/docs/game-sdk/users#oncurrentuserupdate)

## Testing

Unfortunately Discord does not provide a convenient way to perform automated testing, as it requires an actual working Discord application to be running and logged in, which makes automated (particularly headless) testing...annoying.

For now, it's required that you manually spin up 2 different Discord applications (eg, Stable and Canary) and log in with separate accounts on the same machine, then run one test at a time.

### Activities

```sh
cargo test --features local-testing test_activity
```

## Contribution

[![Contributor Covenant](https://img.shields.io/badge/contributor%20covenant-v1.4-ff69b4.svg)](https://github.com/EmbarkStudios/discord-sdk/blob/main/sdk/CODE_OF_CONDUCT.md)

We welcome community contributions to this project.

Please read our [Contributor Guide](https://github.com/EmbarkStudios/discord-sdk/blob/main/sdk/CONTRIBUTING.md) for more information on how to get started.
Please also read our [Contributor Terms](https://github.com/EmbarkStudios/discord-sdk/blob/main/sdk/CONTRIBUTING.md/#Contributor-Terms) before you make any contributions.

Any contribution intentionally submitted for inclusion in an Embark Studios project, shall comply with the Rust standard licensing model (MIT OR Apache 2.0) and therefore be dual licensed as described below, without any additional terms or conditions:

### License

This contribution is dual licensed under EITHER OF

- Apache License, Version 2.0, ([LICENSE-APACHE](https://github.com/EmbarkStudios/discord-sdk/blob/main/sdk/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/EmbarkStudios/discord-sdk/blob/main/sdk/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

For clarity, "your" refers to Embark or any other licensee/user of the contribution.
