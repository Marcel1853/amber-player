# Plugin packages

Sound effects, visualizers and online sources are shared as a **plugin package**: a file ending in
**`.amberplugin`**. A package can contain **one or several parts** – for example three sound
effects and two visualizers – and has **its own license**. Amber Player installs it with one
click, on the PC and on the phone.

Inside, an `.amberplugin` is a regular ZIP file; Amber Player also accepts packages ending in
`.zip`.

## Structure

```
nachtfahrt.amberplugin
├── plugin.json              ← description and list of parts
├── effects/
│   ├── kassette.json        ← sound effect
│   └── stadion.json         ← sound effect
├── visualizers/
│   └── polarlicht.js        ← visualizer
├── LICENSE                  ← the package's license (required)
└── README.md                ← optional: description, instructions
```

Parts may sit directly in the package or in **one** subfolder (e.g. `effects/`). Files other than
`plugin.json`, the parts, `LICENSE` and `README` are not allowed.

## plugin.json

```json
{
  "id": "nachtfahrt",
  "name": "Night drive",
  "version": "1.0.0",
  "author": "Your name",
  "description": "Two sound effects and a visualizer for the night",
  "license": "MIT",
  "homepage": "https://…",
  "contents": [
    { "type": "effect", "file": "effects/kassette.json" },
    { "type": "effect", "file": "effects/stadion.json" },
    { "type": "visualizer", "file": "visualizers/polarlicht.js" }
  ]
}
```

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | 1–32 characters: letters, digits, `-`, `_`; determines the folder name |
| `name` | yes | the package's display name |
| `contents` | yes* | list of parts: `type` (`effect`, `visualizer` or `source`) and `file` (path in the package) – up to 32 parts |
| `permissions` | with `source` | `{ "network": ["api.example.org", "*.example.org"] }` – addresses online sources may reach, see [Creating a source plugin](Creating-a-Source-Plugin.md) |
| `version` | | e.g. `1.0.0` – shown in the settings |
| `author` | | your name |
| `description` | | short description |
| `license` | | short license name, e.g. `MIT`, `GPL-3.0`, `CC-BY-4.0` or “All rights reserved” |
| `homepage` | | where the package can be found |

\* **Short form for a single part:** instead of `contents`, just give `"type": "effect"` (or
`"visualizer"`) and optionally `"main": "effect.json"`.

Each part is built exactly as described in [Creating a sound plugin](Creating-a-Sound-Plugin.md)
and [Creating a visualizer](Creating-a-Visualizer.md). The `id` of every single sound effect and
visualizer must be unique across all installed plugins.

## License

Every package needs its own **LICENSE** file. It applies to all parts in the package. Which
license you choose is up to you:

- **MIT** – anyone may use, modify and share the package as long as your name stays in it. Good
  for templates and anything others should build on.
- **GPL-3.0** – like MIT, but modified versions must stay open too.
- **All rights reserved** – others may use the package, but not modify or redistribute it.

In the settings Amber Player shows the license of every package – and points out when a package
comes without a license file.

## Building a package

The easiest way is the script from the source code – it checks everything Amber Player also
checks during installation and warns about files in the folder that aren't listed in `contents`:

```sh
python3 scripts/plugin-pack.py my-package/        # → my-package.amberplugin
```

Any ZIP program works too: save the folder's files as a ZIP and change the extension to
`.amberplugin` (or keep `.zip`).

## Install, update, remove

- **Install:** Effects → **Install** or Settings → Plugins → **Install**, choose the file.
  Visualizers are available immediately, sound effects after the next app start.
- **Update:** install the new version the same way – a package with the same `id` is replaced
  together with all its parts.
- **Remove:** Settings → Plugins → **Installed plugins** → **Remove**. All parts of the package go
  together.

Packages are installed to `~/.local/share/amber-player/packages/<id>/` (on the phone into the
app's private folder).

## Templates

In the source code under [`examples/plugins/`](https://github.com/Marcel1853/amber-player/tree/main/examples/plugins),
all MIT-licensed; ready-packed files are attached to every
[release](https://github.com/Marcel1853/amber-player/releases):

- `nachtfahrt/` – **several parts:** sound effects “Kassette” and “Stadion”, visualizer “Polarlicht”
- `telefon/` – one sound effect (short form with `type`)
- `sternenfeld/` – one visualizer (short form with `type`)

> Single `.json` and `.js` files without a package still work, but have no version, author or
> license.
