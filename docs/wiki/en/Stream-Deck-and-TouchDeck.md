# Stream Deck and TouchDeck

## Elgato Stream Deck

Each button simply calls an address of the [API](Remote-Control-and-API.md).

1. Install a plugin that calls web addresses – e.g. **API Ninja** or the built-in **Website**
   action with “Open in background”.
2. Enter the address, e.g. `http://127.0.0.1:7878/api/toggle`.

Ready-made addresses to copy are under **Settings → Remote control & MCP → Examples for Stream
Deck buttons**:

| Button | Address |
|---|---|
| Play / Pause | `/api/toggle` |
| Next / Previous track | `/api/next` · `/api/prev` |
| Louder / Quieter | `/api/volume/up` · `/api/volume/down` |
| Volume 30 % | `/api/volume?value=30` |
| Mute | `/api/mute` |
| Shuffle | `/api/shuffle` |
| Favourite (current track) | `/api/favorite` |
| Favourites shuffled | `/api/play/music?favorites=true&shuffle=true` |
| Equalizer on/off | `/api/effects/equalizer/toggle` |
| EQ preset “Bass” | `/api/effects/equalizer?preset=Bass` |

**Cover as button image:** `http://127.0.0.1:7878/api/cover/current` returns the cover of the
current track as an image.

If the Stream Deck is connected to another PC: turn on “Home network” and append `?token=…` to
every address (the examples then already include it).

## TouchDeck

There's a ready-made TouchDeck plugin with a complete player page: title, progress and volume
appear **live** on the buttons.

- Plugin and guide: [`integrations/touchdeck`](https://github.com/Marcel1853/amber-player/tree/main/integrations/touchdeck)
- Ready-made pages in English and German to import

**Setup:**

1. Start Amber Player.
2. Install the plugin in TouchDeck and check the address under **Plugins → Installed**
   (default `http://127.0.0.1:7878`).
3. Import the ready-made page “Amber Player”.

If Amber Player runs on another PC, enter its address and the token in the plugin.

**What the plugin can do:** playback, skip, volume, mute, seek, shuffle, repeat, favourite,
play playlists, play from the library (by artist, album, genre, search term, favourites), switch
sound effects and load presets, sleep timer. Plus triggers like “New track started” for your
own automations.
