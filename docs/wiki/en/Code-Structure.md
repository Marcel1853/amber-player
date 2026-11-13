# Code structure

Amber Player is a Rust workspace with three crates. The same interface and core run on the PC
and on Android.

```
crates/
├── core/            amber-core – everything without a window
├── app/             amber-player – the app (Tauri), PC and Android
└── mobile-plugin/   tauri-plugin-amber-mobile – Android part in Kotlin
```

## crates/core – the core

| Module | Purpose |
|---|---|
| `library.rs` | library in SQLite: tracks, playlists, favourites, history, stations, settings |
| `scanner.rs` | scan music folders, read tags (lofty), clean up titles |
| `player.rs` | audio engine in its own thread: plays the queue, commands, status events |
| `decoder.rs` | audio decoding (Symphonia, minimp3 as fallback for broken MP3s) |
| `httpsource.rs` | seekable HTTP sources, prefetch buffer in its own thread, fragmented MP4 |
| `queue.rs` | queue with shuffle and repeat |
| `effects/` | effect plugins: Rust effects (equalizer, limiter …), JSON effects, safety limiter |
| `analyzer.rs` | frequency bands and waveform for the visualizers |
| `radio.rs`, `netstream.rs` | station search (radio-browser.info) and radio streams with song titles |
| `covers.rs` | online cover search |
| `sources.rs` | source plugins (external programs) |
| `pc.rs` | phone → PC: pairing, requests to the PC API, streaming PC tracks |
| `api.rs`, `mcp.rs` | HTTP API and MCP server (feature `server`, PC only) |

## crates/app – the app

| File | Purpose |
|---|---|
| `src/lib.rs` | app start for PC and Android: data folder, player, plugins, API |
| `src/main.rs` | PC entry point: command line (`--headless`, `--help`) |
| `src/commands.rs` | commands called by the interface (`invoke`) |
| `src/background.rs` | PC: tray icon, quit when no music is playing |
| `src/media.rs` | PC: media keys and MPRIS |
| `ui/index.html`, `ui/style.css`, `ui/app.js` | the interface |
| `ui/i18n.js`, `ui/lang/` | translations, see [Languages and translation](Languages-and-Translation.md) |
| `ui/viz-worker.js`, `ui/visualizers/` | visualizers in a Web Worker |
| `gen/android/` | Android project (Gradle), incl. `MainActivity.kt` (full screen) |

PC-only code is marked with `#[cfg(desktop)]`, Android code with `#[cfg(mobile)]` or
`#[cfg(target_os = "android")]`.

## crates/mobile-plugin – Android

A Tauri plugin with a Kotlin part:

- `AmberMobilePlugin.kt` – commands for the interface: update the notification, permissions,
  battery optimisation, full screen, pairing link from the QR code.
- `PlaybackService.kt` – foreground service with MediaSession: notification, lock screen, media
  buttons, audio focus.

The interface calls it via `invoke('plugin:amber-mobile|…')`; button presses come back as the
`media` event.

## How a track is played

1. The interface calls `play_ids` → `commands.rs` sends `Command::PlayList` to the player.
2. The engine (`player.rs`) loads the track from the library and opens it:
   - local file → `decoder.rs` (with prefetch buffer on the phone),
   - `plugin://pc/…` → stream from the paired PC,
   - `plugin://<source>/…` → address from the source plugin.
3. The sound passes through the **effect chain** (`effects/`), the **analyzer** and the **safety
   limiter** to the output (rodio/cpal; on the phone with a larger buffer via Oboe).
4. The engine reports its status; the app forwards it to the interface as the `status` event.

## Data

| | Location |
|---|---|
| library and settings | `~/.local/share/amber-player/library.db` (Android: private app folder) |
| plugins | `~/.local/share/amber-player/{effects,visualizers,sources}/` |
| downloaded covers | `~/.local/share/amber-player/covers/` |

## Tests

```sh
cargo test --workspace
```

Tests cover the queue, library, JSON effects and formulas, decoder, HTTP sources, API ranges and
PC pairing, among others.
