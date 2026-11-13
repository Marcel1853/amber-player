# Creating plugins

You can extend Amber Player without changing or rebuilding the player itself.

| I want … | Plugin kind | Skills | Guide |
|---|---|---|---|
| my own sound (filters, echo, “telephone”, “stadium” …) | sound effect | none – just fill in JSON | [Creating a sound plugin](Creating-a-Sound-Plugin.md) |
| my own visualizer | visualizer | some JavaScript (Canvas 2D) | [Creating a visualizer](Creating-a-Visualizer.md) |
| to add music from an online service | source | any program, e.g. Python | [Creating a source plugin](Creating-a-Source-Plugin.md) |

## How to proceed

1. **Copy a template:** [`examples/plugins/`](https://github.com/Marcel1853/amber-player/tree/main/examples/plugins)
   or one of the included plugins.
2. **Change `id` and `name`.** The `id` must be unique.
3. **Put it in the plugin folder** or import it on the phone – see [Installing plugins](Installing-Plugins.md).
4. **Try it:** sound effects after a restart, visualizers immediately via **Reload**.
5. **Read errors:** sound effects show them on the Effects page, visualizers at the bottom right.

## Included plugins as examples

| What | Where in the source code |
|---|---|
| Sound effects in JSON format (Bass booster, Echo, Old radio …) | [`crates/core/src/effects/builtin/`](https://github.com/Marcel1853/amber-player/tree/main/crates/core/src/effects/builtin) |
| Visualizers (Bars, Wave, Ring, Fog) | [`crates/app/ui/visualizers/`](https://github.com/Marcel1853/amber-player/tree/main/crates/app/ui/visualizers) |
| Built-in effects in Rust (equalizer, limiter …) | [`crates/core/src/effects/`](https://github.com/Marcel1853/amber-player/tree/main/crates/core/src/effects) |

## Sharing

To share your plugin, pack it as a **plugin package**: a ZIP file with `plugin.json` (name,
version, author, license), the plugin file and its own **LICENSE**. How it works and which
license fits: [Plugin packages](Plugin-Packages.md).

The plugin templates in `examples/plugins/` are MIT-licensed, and the code examples in the documentation may be
used freely for your own plugins (see [LICENSE](https://github.com/Marcel1853/amber-player/blob/main/LICENSE)).

The complete reference with all fields is in
[`docs/PLUGINS.md`](https://github.com/Marcel1853/amber-player/blob/main/docs/PLUGINS.md) (German).
