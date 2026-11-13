# Plugins erstellen

Du kannst Amber Player erweitern, ohne den Player selbst zu verändern oder neu zu bauen.

| Ich möchte … | Plugin-Art | Kenntnisse | Anleitung |
|---|---|---|---|
| einen eigenen Klang (Filter, Echo, „Telefon“, „Stadion“ …) | Klangeffekt | keine – nur JSON ausfüllen | [Klang-Plugin erstellen](Klang-Plugin-erstellen.md) |
| eine eigene Visualisierung | Visualisierung | etwas JavaScript (Canvas 2D) | [Visualisierung erstellen](Visualisierung-erstellen.md) |
| Musik aus einem Online-Dienst einbinden | Quelle | ein beliebiges Programm, z. B. Python | [Quellen-Plugin erstellen](Quellen-Plugin-erstellen.md) |

## So gehst du vor

1. **Vorlage kopieren:** [`examples/plugins/`](https://github.com/Marcel1853/amber-player/tree/main/examples/plugins)
   oder eins der mitgelieferten Plugins.
2. **`id` und `name` ändern.** Die `id` muss eindeutig sein.
3. **In den Plugin-Ordner legen** bzw. am Handy importieren – siehe [Plugins installieren](Plugins-installieren.md).
4. **Ausprobieren:** Klangeffekte nach Neustart, Visualisierungen sofort über **Neu laden**.
5. **Fehler lesen:** Klangeffekte zeigen sie auf der Seite Effekte, Visualisierungen unten rechts.

## Mitgelieferte Plugins als Vorbild

| Was | Wo im Quellcode |
|---|---|
| Klangeffekte im JSON-Format (Bass-Booster, Echo, Altes Radio …) | [`crates/core/src/effects/builtin/`](https://github.com/Marcel1853/amber-player/tree/main/crates/core/src/effects/builtin) |
| Visualisierungen (Balken, Welle, Ring, Nebel) | [`crates/app/ui/visualizers/`](https://github.com/Marcel1853/amber-player/tree/main/crates/app/ui/visualizers) |
| Fest eingebaute Effekte in Rust (Equalizer, Limiter …) | [`crates/core/src/effects/`](https://github.com/Marcel1853/amber-player/tree/main/crates/core/src/effects) |

## Weitergeben

Zum Weitergeben packst du dein Plugin als **Plugin-Paket**: eine ZIP-Datei mit `plugin.json`
(Name, Version, Autor, Lizenz), der Plugin-Datei und einer eigenen **LICENSE**. Wie das geht
und welche Lizenz passt: [Plugin-Pakete](Plugin-Pakete.md).

Das vollständige Nachschlagewerk mit allen Feldern steht in
[`docs/PLUGINS.md`](https://github.com/Marcel1853/amber-player/blob/main/docs/PLUGINS.md).
