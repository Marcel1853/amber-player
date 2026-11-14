# Installing plugins

Sound effects, visualizers and online sources come as a **plugin package** – an `.amberplugin`
file with one or more parts and its own license, see [Plugin packages](Plugin-Packages.md). The
quickest way is the **Install plugin** button (at the top of Effects and under Settings →
Plugins): it accepts any package, whatever it contains – the same on the PC and the phone. When
the phone is connected to the PC, you can also get packages under **PC → Plugins** straight from
the PC.

Amber Player knows three kinds of plugins:

| Kind | File | Where | Takes effect |
|---|---|---|---|
| **Sound effect** | package (`.amberplugin`) or one `.json` file | PC and phone | after restarting the app |
| **Visualizer** | package (`.amberplugin`) or one `.js` file | PC and phone | immediately (PC: after “Reload”) |
| **Online source** (online music) | package (`.amberplugin`) with JavaScript | PC and phone | after restart, if “Extended plugins” is on |
| **Program source** (older kind) | a folder with `plugin.json` and a program | PC only | after restart |

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

1. Get the package (`.amberplugin`) or the `.json` / `.js` file onto the phone (download, messenger, cable …).
2. **Effects → Install** (sound effects) or **Settings → Plugins → Install**.
3. Choose the file.

Amber Player checks sound effects while importing; a broken file is rejected and the error is
shown. Sound effects take effect after the next app start, visualizers immediately.

Source plugins aren't available on the phone.

## Removing plugins

Packages: **Settings → Plugins → Installed plugins → Remove**. Single files: delete them and
restart Amber Player. On the phone: clearing the app data removes
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

- `nachtfahrt/` – package with several parts: two sound effects and a visualizer (MIT license)
- `telefon/` – sound effect “Telephone” (package, MIT license)
- `sternenfeld/` – visualizer “Star field” (package, MIT license)

Build your own: [Creating plugins](Creating-Plugins.md)
