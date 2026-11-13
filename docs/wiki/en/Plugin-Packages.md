# Plugin packages

A sound effect or visualizer is shared as a **plugin package**: a ZIP file with a description,
the plugin itself and **its own license**. Amber Player installs the package with one click –
on the PC and on the phone.

## Structure

```
telephone.zip
├── plugin.json     ← description: name, kind, version, author, license
├── effect.json     ← the plugin (for visualizers: visualizer.js)
├── LICENSE         ← this plugin's license (required)
└── README.md       ← optional: description, instructions
```

The files may sit directly in the ZIP file or in a shared folder (`telephone/plugin.json` …).
Other files are not allowed.

## plugin.json

```json
{
  "id": "telephone",
  "name": "Telephone",
  "type": "effect",
  "version": "1.0.0",
  "author": "Your name",
  "description": "Sounds like a phone call",
  "license": "MIT",
  "main": "effect.json",
  "homepage": "https://…"
}
```

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | 1–32 characters: letters, digits, `-`, `_`; determines the folder name |
| `name` | yes | display name |
| `type` | yes | `effect` (sound effect) or `visualizer` |
| `version` | | e.g. `1.0.0` – shown in the settings |
| `author` | | your name |
| `description` | | short description |
| `license` | | short license name, e.g. `MIT`, `GPL-3.0`, `CC-BY-4.0` or “All rights reserved” |
| `main` | | plugin file; default `effect.json` or `visualizer.js` |
| `homepage` | | where the plugin can be found |

The plugin file itself (`effect.json` or `visualizer.js`) is built exactly as described in
[Creating a sound plugin](Creating-a-Sound-Plugin.md) and [Creating a visualizer](Creating-a-Visualizer.md).

## License

Every package needs its own **LICENSE** file. Which license you choose is up to you:

- **MIT** – anyone may use, modify and share the plugin as long as your name stays in it. Good
  for templates and anything others should build on.
- **GPL-3.0** – like MIT, but modified versions must stay open too.
- **All rights reserved** – others may use the plugin, but not modify or redistribute it.

In the settings Amber Player shows the license of every package – and points out when a
package comes without a license file.

## Building a package

The easiest way is the script from the source code – it checks everything Amber Player also
checks during installation:

```sh
python3 scripts/plugin-pack.py my-plugin/        # → my-plugin.zip
```

Any ZIP program works too: select the files in the folder and save them as a ZIP.

## Install, update, remove

- **Install:** Effects → **Install** or Settings → Plugins → **Install**, choose the ZIP file.
  Sound effects take effect after the next app start, visualizers immediately.
- **Update:** install the new version the same way – a package with the same `id` is replaced.
- **Remove:** Settings → Plugins → **Installed plugins** → **Remove**.

Each package gets its own folder: `~/.local/share/amber-player/effects/<id>/` or
`…/visualizers/<id>/` (on the phone in the app's private folder).

## Templates

The source code contains two ready-made packages (MIT license) in
[`examples/plugins/`](https://github.com/Marcel1853/amber-player/tree/main/examples/plugins);
ready-packed ZIPs are attached to every [release](https://github.com/Marcel1853/amber-player/releases):

- `telefon/` – sound effect
- `sternenfeld/` – visualizer

> Single `.json` and `.js` files without a package still work, but have no version, author or
> license.
