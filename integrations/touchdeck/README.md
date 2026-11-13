# Amber Player for TouchDeck

**English** · [Deutsch](README.de.md)

Control the **Amber Player** music player from your TouchDeck buttons: play/pause, skip, volume,
shuffle, repeat, favourites, playlists, equalizer presets and sleep timer. The current song,
progress and volume are shown live on your buttons.

Regular plugin (JavaScript, sandboxed). It only talks to Amber Player on this PC or in your home
network (`lan`); nothing goes to the internet.

## Setup

1. Start Amber Player. Its API runs on `http://127.0.0.1:7878` automatically.
2. Install the plugin and check the address under **Plugins › Installed**.
3. Import the ready-made page “Amber Player” (plugin card › Ready-made pages).

If Amber Player runs on another PC: enable “In home network” under **Settings › Remote control**
there, restart Amber Player and enter its address (e.g. `http://192.168.1.20:7878`) and the token
in the plugin settings.

## Actions

Play/pause, play, pause, stop, next/previous song, volume (set, louder, quieter), mute, jump in
song, shuffle, repeat, favourite for the current song, play playlist (by name or ID), play from
library (artist, album, genre, search term, favourites only), sound effects (equalizer,
crossfeed, stereo width, limiter: on, off, toggle, load preset), sleep timer and “current song
as data”.

## Live displays

Now playing, title, artist, album, progress (bar), playing, volume (bar), shuffle, repeat,
favourite, equalizer and sleep timer.

## Triggers

- **New song started** – with `{Event.title}`, `{Event.artist}`, `{Event.album}`
- **Playback started / paused / stopped** – with `{Event.state}`

## Ready-made page

Rebuild with `python3 tools/make_profile.py`.

## More

All API endpoints: [docs/API.md](../../docs/API.md) (German) · Manual: [Stream Deck and TouchDeck](../../docs/wiki/Stream-Deck-und-TouchDeck.md) (German)

## License

MIT
