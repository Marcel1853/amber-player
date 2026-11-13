# Install

## Linux (PC)

Amber Player comes in three package formats. Pick the one for your distribution:

| Package | For | How to install |
|---|---|---|
| `Amber Player_<version>_amd64.deb` | Ubuntu, Debian, Linux Mint, Pop!_OS, Zorin | Double-click (opens the software centre) or in a terminal: `sudo apt install "./Amber Player_<version>_amd64.deb"` |
| `Amber Player-<version>-1.x86_64.rpm` | Fedora, openSUSE, Nobara | `sudo dnf install "./Amber Player-<version>-1.x86_64.rpm"` |
| `Amber Player_<version>_amd64.AppImage` | any distribution, no installation | `chmod +x "Amber Player_<version>_amd64.AppImage"` and double-click |

Afterwards you'll find **Amber Player** in your application menu.

### Updating

Install the new package the same way – library, playlists and settings are kept. They live
separately from the program in `~/.local/share/amber-player/`.

### Uninstalling

```sh
sudo apt remove amber-player        # or: sudo dnf remove amber-player
rm -rf ~/.local/share/amber-player  # only if library and settings should go too
```

### Closing and background

- **No music playing:** closing the window quits Amber Player completely – nothing stays in the
  background.
- **Music playing:** it keeps playing; a tray icon offers Open, Play/Pause, Next and Quit. When
  the music ends or stays paused for a minute, Amber Player quits by itself.

## Android (phone and tablet)

1. Download `AmberPlayer-<version>.apk` to the device (browser, cable …).
2. Tap it. The first time, Android asks whether apps from this source may be installed – allow it
   for your browser or file manager.
3. Tap **Install**.
4. On first start the app asks for:
   - **Access to music and audio** – needed to read your music,
   - **Notifications** – for the playback controls in the notification shade,
   - **Exemption from battery optimisation** – otherwise some manufacturers (e.g. Samsung) stop
     the music in the background.

Requirements: Android 7 or newer, 64-bit device (arm64 – practically every phone and tablet from
recent years).

### Updating

Install the new APK on top. Library and settings are kept.

> **Samsung “Dual Messenger”:** If Amber Player shows up twice, it was also installed in the Dual
> Messenger profile. Simply uninstall one of the copies.

## Without a window (server, Raspberry Pi on the TV)

Amber Player also runs without an interface – playback, HTTP API and MCP server only:

```sh
amber-player --headless --music ~/Music
amber-player --headless --music /mnt/nas/music --lan --port 7878
```

| Option | Meaning |
|---|---|
| `--music <folder>` | add a music folder (can be repeated) |
| `--port <port>` | API port (default 7878) |
| `--lan` | make the API available on your home network; the token is shown at start |

Control it via the [API](Remote-Control-and-API.md), a [Stream Deck or TouchDeck](Stream-Deck-and-TouchDeck.md)
or the [phone app](Connecting-to-your-PC.md).

Next: [Getting started](Getting-Started.md)
