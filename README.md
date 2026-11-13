**English** · [Deutsch](README.de.md)

# Amber Player

**A music player for Linux and Android** – fast, good-looking, free and without ads.
Written in Rust (Tauri 2, rodio), with its own interface in a warm dark design.

---

## What Amber Player can do

**Listening**
- **Your own library:** choose your music folders – Amber Player scans them and keeps them up to
  date automatically. MP3, FLAC, OGG/Opus, WAV, M4A/AAC, AIFF and more.
- **Views** for tracks, albums, artists, genres, favourites, recently played and most played,
  plus live search.
- **Playlists:** create, reorder, import and export as M3U.
- **Queue** with “Play next”, shuffle, repeat and a sleep timer. On the next start it continues
  where you left off.
- **Covers** from the file, from `cover.jpg`/`folder.jpg` or automatically from the internet
  (Deezer, MusicBrainz – only on a confident match, can be turned off).
- **Internet radio:** search more than 50,000 stations (radio-browser.info), save your own,
  song titles and station logos are shown.

**Sound and visuals**
- **Sound effects as plugins:** 10-band equalizer with presets, bass booster, clearer voices,
  headphone crossfeed, stereo width, echo, old radio, 8D flight, preamp & limiter. A safety
  limiter prevents clipping and crackling.
- **Visualizers** in full screen: Bars, Wave, Ring (with a spinning cover) and Fog.
- **Your own plugins** without rebuilding the player: sound effects as a JSON file (no
  programming), visualizers as a JavaScript file.

**Phone and PC together**
- **Android app** with everything for listening: library, playlists, effects, visualizers,
  radio. Background playback with notification, lock screen and headphone/Bluetooth buttons.
- **Pair with your PC** (scan a QR code): control the PC from your phone or stream music and
  playlists from the PC to the phone – without copying files.
- **Remote control** via an HTTP API (Stream Deck, TouchDeck, Home Assistant, curl) and a
  built-in **MCP server** for AI assistants like Claude.

**At home everywhere**
- **Eight languages:** English, German, Spanish, French, Italian, Dutch, Polish, Turkish –
  automatically by system language or set manually.
- **Adapts:** PC window, tablet in portrait and landscape, phone in portrait and landscape.
- **Design:** dark or light, accent colour of your choice.
- **Free and without ads** – now and in the future.

## Install

### Linux (PC)

| Package | For | How to install |
|---|---|---|
| `.deb` | Ubuntu, Debian, Linux Mint, Pop!_OS | double-click or `sudo apt install "./Amber Player_<version>_amd64.deb"` |
| `.rpm` | Fedora, openSUSE | `sudo dnf install "./Amber Player-<version>-1.x86_64.rpm"` |
| `.AppImage` | any distribution | `chmod +x` and start |

Afterwards Amber Player is in your application menu. On first start click **Choose music
folder** – done.

### Android

Download the APK file to your phone and open it (allow “install from this source” once). On
first start the app asks for access to your music and for notifications, then scans the
**Music** folder. Android 7 or newer, 64-bit.

Details: [Install](docs/wiki/en/Install.md) · [Phone app](docs/wiki/en/Phone-App.md)

## Quick start

| I want to … | How |
|---|---|
| play music | tap or double-click a track, or **Play** / **Shuffle** |
| create a playlist | **Playlists → New playlist**, add tracks via right-click (phone: long-press) |
| change the sound | sliders icon at the top → **Effects** |
| listen to radio | **Radio** → search a station → tap it, save with ★ |
| pair my phone with the PC | on the PC **Settings → Remote control → Pair phone**, scan the QR code |
| change the language | **Settings → Appearance → Language** |
| build my own plugin | [Creating plugins](docs/wiki/en/Creating-Plugins.md) |
| know my version | at the very bottom of **Settings** |

## Documentation

The full manual is in the **wiki** – in [English](docs/wiki/en/English.md) and
[German](docs/wiki/Home.md) (also on [GitHub Wiki](https://github.com/Marcel1853/amber-player/wiki)):

- [Getting started](docs/wiki/en/Getting-Started.md) · [Usage and shortcuts](docs/wiki/en/Usage.md)
- [Phone app](docs/wiki/en/Phone-App.md) · [Connecting to your PC](docs/wiki/en/Connecting-to-your-PC.md)
- [Internet radio](docs/wiki/en/Internet-Radio.md) · [Sound effects](docs/wiki/en/Sound-Effects.md) · [Visualizers](docs/wiki/en/Visualizers.md)
- [Installing plugins](docs/wiki/en/Installing-Plugins.md) · [Creating plugins](docs/wiki/en/Creating-Plugins.md)
- [Remote control and API](docs/wiki/en/Remote-Control-and-API.md) · [Stream Deck and TouchDeck](docs/wiki/en/Stream-Deck-and-TouchDeck.md) · [MCP and AI assistants](docs/wiki/en/MCP-and-AI.md)
- [Languages and translation](docs/wiki/en/Languages-and-Translation.md) · [Building from source](docs/wiki/en/Building-from-Source.md) · [FAQ](docs/wiki/en/FAQ.md)

References for developers (German):

- [docs/PLUGINS.md](docs/PLUGINS.md) – all plugin interfaces in detail
- [docs/API.md](docs/API.md) – all endpoints of the HTTP API and the MCP server
- [integrations/touchdeck](integrations/touchdeck/README.md) – TouchDeck plugin with a ready-made page

## Building (short version)

```sh
# Requirements (Ubuntu/Debian): Rust via https://rustup.rs and
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
                 librsvg2-dev libasound2-dev libdbus-1-dev

cargo test --workspace                     # tests
cargo run -p amber-player                  # start (development)
cargo install tauri-cli --version "^2" --locked
cd crates/app && cargo tauri build         # .deb, .rpm, AppImage → target/release/bundle/
bash scripts/android-apk.sh                # Android app → target/android/
```

Everything else, including Android: [Building from source](docs/wiki/en/Building-from-Source.md).

## Structure

```
crates/core/            heart of the player: library (SQLite), scanner, queue, audio engine,
                        effects, radio, PC connection, HTTP API, MCP server
crates/app/             app for PC and Android (Tauri)
  ui/                   interface (HTML/CSS/JavaScript), visualizers, language files
  i18n/                 translation tables → ui/lang/*.js
  gen/android/          Android project
crates/mobile-plugin/   Android part: background playback, notification, media buttons
examples/plugins/       templates for your own plugins
integrations/touchdeck/ TouchDeck plugin
docs/                   manual (wiki/, wiki/en/) and references
scripts/                helper scripts (build the Android app, upload the wiki)
```

## License

**All rights reserved.** Amber Player is free to use but not open source: the source code is
visible, but may not be copied, modified or redistributed as your own version. Your own plugins
are explicitly welcome – each plugin package carries its own license; the templates in
`examples/plugins/` are MIT. Details: [LICENSE](LICENSE).

## Privacy

Amber Player collects no data and shows no ads. Only what you turn on goes online: the cover
search (artist, title, album), the radio station list, the radio streams themselves and – once a
day, can be turned off – the update check on GitHub. The HTTP API is only reachable from your own
PC by default.
