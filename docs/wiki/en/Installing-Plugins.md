# Installing plugins

Amber Player knows three kinds of plugins:

| Kind | File | Where | Takes effect |
|---|---|---|---|
| **Sound effect** | one `.json` file | PC and phone | after restarting the app |
| **Visualizer** | one `.js` file | PC and phone | immediately (PC: after “Reload”) |
| **Source** (online music) | a folder with `plugin.json` and a program | PC only | after restart |

> **Security:** Sound effects are pure descriptions without program code. Visualizers run
> sandboxed without network or file access. **Source plugins, however, are programs** with the
> same rights as you – only install them from sources you trust.

## On the PC (Linux)

The plugin folders are in `~/.local/share/amber-player/`:

```
~/.local/share/amber-player/
├── effects/          ← sound effects (*.json)
├── visualizers/      ← visualizers (*.js)
└── sources/          ← source plugins (one subfolder each)
```

The easiest way to open them is from the program:

- **Effects → Open plugin folder**
- **Settings → Plugins → Open folder** (one per kind)

Copy the file in, then

- for **sound effects** restart Amber Player,
- for **visualizers** click **Reload** under Settings → Plugins,
- for **source plugins** copy the whole folder (e.g. `sources/my-source/`) and restart. A new
  **Online** tab appears.

## On the phone (Android)

The phone gives no access to the app folder, so:

1. Get the `.json` or `.js` file onto the phone (download, messenger, cable …).
2. **Effects → Import** (sound effects) or **Settings → Plugins → Import**.
3. Choose the file.

Amber Player checks sound effects while importing; a broken file is rejected and the error is
shown. Sound effects take effect after the next app start, visualizers immediately.

Source plugins aren't available on the phone.

## Removing plugins

Delete the file or folder and restart Amber Player. On the phone: clearing the app data removes
all imported plugins (but also library and settings).

## If a plugin doesn't show up

- **Sound effect:** the top of the **Effects** page lists under “Not loaded” what's wrong – e.g.
  a typo in the JSON or a duplicate `id`.
- **Visualizer:** errors appear as a message at the bottom right. The file may be at most 512 KB
  and must end in `.js`.
- **Source:** errors are shown on the **Online** page; the program must be startable from the
  plugin folder (e.g. `python3` installed).

## Templates

Examples to try in the source code: [`examples/plugins/`](https://github.com/Marcel1853/amber-player/tree/main/examples/plugins)

- `telefon.json` – sound effect “Telephone”
- `sternenfeld.js` – visualizer “Star field”

Build your own: [Creating plugins](Creating-Plugins.md)
